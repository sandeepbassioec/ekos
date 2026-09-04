//! O_DIRECT aligned writer — bypasses the page cache for predictable write
//! latency (VV-STO-009 Linux).
//!
//! On Linux the file is opened with `O_DIRECT`, which requires both the buffer
//! pointer and the transfer size to be 512-byte aligned.  Use [`DirectWriter::aligned_buf`]
//! to obtain a correctly aligned [`AlignedBuf`], then write it with
//! [`DirectWriter::write_aligned`].
//!
//! On Windows and macOS the fallback path opens a regular file; the alignment
//! requirement is relaxed so the same code compiles and tests pass on all
//! platforms.

use std::alloc::Layout;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

/// Alignment required for O_DIRECT I/O (512 bytes on all Linux block devices).
pub const DIRECT_ALIGN: usize = 512;

/// Round `n` up to the next multiple of `align`.
#[inline]
pub fn align_up(n: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (n + align - 1) & !(align - 1)
}

// ── AlignedBuf ────────────────────────────────────────────────────────────────

/// A heap-allocated buffer whose pointer and length are both multiples of
/// [`DIRECT_ALIGN`] (512 bytes).
///
/// This type owns its allocation and uses the correct layout on drop, avoiding
/// the heap corruption that arises from returning a `Vec<u8>` allocated with a
/// non-standard alignment.
pub struct AlignedBuf {
    ptr: NonNull<u8>,
    len: usize,
    layout: Layout,
}

// SAFETY: AlignedBuf owns its allocation and has no thread-local state.
unsafe impl Send for AlignedBuf {}
unsafe impl Sync for AlignedBuf {}

impl AlignedBuf {
    /// Allocate at least `size` bytes, rounded up to [`DIRECT_ALIGN`], zeroed.
    pub fn new(size: usize) -> Self {
        let rounded = if size == 0 { DIRECT_ALIGN } else { align_up(size, DIRECT_ALIGN) };
        let layout =
            Layout::from_size_align(rounded, DIRECT_ALIGN).expect("AlignedBuf: layout overflow");
        // SAFETY: layout.size() > 0.
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            panic!("AlignedBuf: allocation of {rounded} bytes failed");
        }
        // SAFETY: ptr is non-null.
        Self { ptr: unsafe { NonNull::new_unchecked(ptr) }, len: rounded, layout }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr is valid for len bytes.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: &mut self is exclusive access.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for AlignedBuf {
    fn drop(&mut self) {
        // SAFETY: ptr and layout came from alloc_zeroed; still live (single ownership).
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), self.layout) };
    }
}

impl std::ops::Index<usize> for AlignedBuf {
    type Output = u8;
    fn index(&self, i: usize) -> &u8 {
        &self.as_slice()[i]
    }
}

impl std::ops::IndexMut<usize> for AlignedBuf {
    fn index_mut(&mut self, i: usize) -> &mut u8 {
        &mut self.as_mut_slice()[i]
    }
}

// ── DirectWriter ──────────────────────────────────────────────────────────────

/// O_DIRECT file writer — bypasses page cache for predictable write latency.
///
/// On Linux buffers must be multiples of 512 bytes and the pointer must be
/// 512-byte aligned; use [`aligned_buf`](Self::aligned_buf) to allocate them.
/// On non-Linux platforms any buffer is accepted (regular I/O).
pub struct DirectWriter {
    #[cfg(target_os = "linux")]
    file: std::fs::File,
    #[cfg(not(target_os = "linux"))]
    file: io::BufWriter<std::fs::File>,
    path: PathBuf,
    pos: u64,
}

impl DirectWriter {
    /// Open `path` for O_DIRECT writing (Linux) or regular writing (other platforms).
    pub fn open(path: &Path) -> io::Result<Self> {
        Self::open_impl(path)
    }

    #[cfg(target_os = "linux")]
    fn open_impl(path: &Path) -> io::Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .custom_flags(libc::O_DIRECT)
            .open(path)?;
        Ok(Self { file, path: path.to_owned(), pos: 0 })
    }

    #[cfg(not(target_os = "linux"))]
    fn open_impl(path: &Path) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        Ok(Self { file: io::BufWriter::new(file), path: path.to_owned(), pos: 0 })
    }

    /// Write a slice.
    ///
    /// On Linux `buf.len()` must be a multiple of [`DIRECT_ALIGN`]; use
    /// [`aligned_buf`](Self::aligned_buf) to ensure this.
    pub fn write_aligned(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let n = self.file.write(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    /// Allocate a 512-byte-aligned buffer of at least `size` bytes (rounded up
    /// to [`DIRECT_ALIGN`]).
    pub fn aligned_buf(size: usize) -> AlignedBuf {
        AlignedBuf::new(size)
    }

    /// Current write position (bytes written so far).
    pub fn pos(&self) -> u64 {
        self.pos
    }

    /// Flush kernel buffers to durable storage.
    pub fn fsync(&mut self) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::io::AsRawFd;
            let rc = unsafe { libc::fsync(self.file.as_raw_fd()) };
            if rc != 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
        }
        #[cfg(not(target_os = "linux"))]
        {
            self.file.flush()
        }
    }

    /// Return the path this writer was opened on.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn tmp_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        static CTR: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = CTR.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        p.push(format!("vv_direct_test_{}_{}.bin", std::process::id(), n));
        p
    }

    // 1 — open creates the file
    #[test]
    fn direct_open_creates_file() {
        let p = tmp_path();
        let _w = DirectWriter::open(&p).expect("open");
        assert!(p.exists());
        let _ = std::fs::remove_file(&p);
    }

    // 2 — write_aligned 512 bytes
    #[test]
    fn direct_write_512() {
        let p = tmp_path();
        let mut w = DirectWriter::open(&p).expect("open");
        let buf = DirectWriter::aligned_buf(512);
        let n = w.write_aligned(buf.as_slice()).expect("write");
        assert_eq!(n, 512);
        let _ = std::fs::remove_file(&p);
    }

    // 3 — write_aligned 1024 bytes
    #[test]
    fn direct_write_1024() {
        let p = tmp_path();
        let mut w = DirectWriter::open(&p).expect("open");
        let buf = DirectWriter::aligned_buf(1024);
        let n = w.write_aligned(buf.as_slice()).expect("write");
        assert_eq!(n, 1024);
        let _ = std::fs::remove_file(&p);
    }

    // 4 — aligned_buf size is multiple of 512
    #[test]
    fn direct_aligned_buf_multiple_of_512() {
        for size in [1usize, 511, 512, 513, 1000, 2048] {
            let b = DirectWriter::aligned_buf(size);
            assert_eq!(b.len() % DIRECT_ALIGN, 0, "size {size} not 512-aligned");
        }
    }

    // 5 — aligned_buf ptr is 512-aligned
    #[test]
    fn direct_aligned_buf_ptr_aligned() {
        let b = DirectWriter::aligned_buf(1024);
        assert_eq!(b.as_ptr() as usize % DIRECT_ALIGN, 0, "ptr not aligned");
    }

    // 6 — write then read back correct
    #[test]
    fn direct_write_read_roundtrip() {
        let p = tmp_path();
        let mut w = DirectWriter::open(&p).expect("open");
        let mut buf = DirectWriter::aligned_buf(512);
        buf[0] = 0xDE;
        buf[511] = 0xAD;
        w.write_aligned(buf.as_slice()).expect("write");
        w.fsync().expect("fsync");
        drop(w);
        let mut f = std::fs::File::open(&p).expect("reopen");
        let mut read_buf = vec![0u8; 512];
        f.read_exact(&mut read_buf).expect("read");
        assert_eq!(read_buf[0], 0xDE);
        assert_eq!(read_buf[511], 0xAD);
        let _ = std::fs::remove_file(&p);
    }

    // 7 — pos advances after write
    #[test]
    fn direct_pos_advances() {
        let p = tmp_path();
        let mut w = DirectWriter::open(&p).expect("open");
        assert_eq!(w.pos(), 0);
        w.write_aligned(DirectWriter::aligned_buf(512).as_slice()).expect("write");
        assert_eq!(w.pos(), 512);
        let _ = std::fs::remove_file(&p);
    }

    // 8 — fsync ok
    #[test]
    fn direct_fsync_ok() {
        let p = tmp_path();
        let mut w = DirectWriter::open(&p).expect("open");
        w.write_aligned(DirectWriter::aligned_buf(512).as_slice()).expect("write");
        w.fsync().expect("fsync");
        let _ = std::fs::remove_file(&p);
    }

    // 9 — write 0 bytes no-op
    #[test]
    fn direct_write_zero_noop() {
        let p = tmp_path();
        let mut w = DirectWriter::open(&p).expect("open");
        let n = w.write_aligned(&[]).expect("write empty");
        assert_eq!(n, 0);
        assert_eq!(w.pos(), 0);
        let _ = std::fs::remove_file(&p);
    }

    // 10 — multiple writes accumulate pos
    #[test]
    fn direct_multiple_writes_accumulate_pos() {
        let p = tmp_path();
        let mut w = DirectWriter::open(&p).expect("open");
        for _ in 0..4 {
            w.write_aligned(DirectWriter::aligned_buf(512).as_slice()).expect("write");
        }
        assert_eq!(w.pos(), 2048);
        let _ = std::fs::remove_file(&p);
    }

    // 11 — aligned_buf 1 byte rounds to 512
    #[test]
    fn direct_aligned_buf_1_rounds_to_512() {
        let b = DirectWriter::aligned_buf(1);
        assert_eq!(b.len(), 512);
    }

    // 12 — aligned_buf 513 bytes rounds to 1024
    #[test]
    fn direct_aligned_buf_513_rounds_to_1024() {
        let b = DirectWriter::aligned_buf(513);
        assert_eq!(b.len(), 1024);
    }

    // 13 — open nonexistent dir returns Err
    #[test]
    fn direct_open_nonexistent_dir_returns_err() {
        let p = PathBuf::from("/nonexistent_vv_dir_xyz/test.bin");
        assert!(DirectWriter::open(&p).is_err());
    }
}
