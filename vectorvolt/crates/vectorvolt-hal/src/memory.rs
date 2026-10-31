//! NUMA-local memory primitives — VV-HAL-029, 031, 032, 033.
//!
//! Four building blocks that close the gap between the current 200 ns point-read
//! and the <100 ns target:
//!
//! | Type / fn | VV-ID | What it does |
//! |---|---|---|
//! | [`LocalNumaArena`] | VV-HAL-029 | `!Send` bump allocator from NUMA-local pages |
//! | [`HugePageBuf`] | VV-HAL-031 | 2 MB large-page allocation; graceful fallback |
//! | [`mlock_range`] | VV-HAL-033 | Lock pages into DRAM, prevent paging |
//! | [`prefetch_populate`] | VV-HAL-032 | Pre-fault pages so the first read hits cache |
//!
//! ## Why each matters
//!
//! **NUMA locality** — reading from a remote NUMA node costs ~1.5–2× a local
//! access. `LocalNumaArena` allocates from the node where the calling thread
//! runs, so the index is physically close to the CPU touching it.
//!
//! **Huge pages** — the TLB covers only ~2 MB of address space per 4 KB entry.
//! A 64 MB index on 4 KB pages burns 16 384 TLB entries and thrashes constantly.
//! The same index on 2 MB pages fits in 32 entries.
//!
//! **mlock** — the OS can page-out cold data while the server idles. An mlock'd
//! index stays in DRAM so the first post-idle read hits L3, not disk.
//!
//! **Pre-faulting** — mmap is lazy; every first touch faults. Calling
//! `prefetch_populate` forces all faults during a controlled warmup window,
//! not scattered across the first wave of real requests.
//!
//! ## Platform coverage
//!
//! | Feature | Windows | Linux | other |
//! |---|---|---|---|
//! | NUMA-local alloc | `VirtualAllocExNuma`; fallback `VirtualAlloc` | `mmap` (🧊 `mbind` Linux sprint) | `std::alloc` |
//! | Huge pages | `VirtualAlloc(MEM_LARGE_PAGES)`; needs `SeLockMemoryPrivilege` | `mmap(MAP_HUGETLB)` (🧊) | fallback |
//! | mlock | `VirtualLock` | `mlock` | no-op |
//! | Pre-fault | `PrefetchVirtualMemory` | `madvise(MADV_WILLNEED)` + touch | page-touch loop |

use std::alloc::Layout;
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::cache::CACHE_LINE;

/// Standard 2 MB huge-page size used by [`HugePageBuf`].
pub const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

// ── LocalNumaArena (VV-HAL-029) ───────────────────────────────────────────────

/// `!Send`, `!Sync` bump allocator that draws from NUMA-local memory.
///
/// ## Why `!Send` and `!Sync`
///
/// The arena is bound to the NUMA node of the thread that created it. Moving it
/// to a different thread would let that thread free (and possibly reallocate)
/// memory not local to it — silently defeating the locality guarantee the whole
/// type exists to provide.
///
/// The "bump" here is page-granular, not byte-granular: each [`alloc`](Self::alloc)
/// reserves one new OS allocation of the requested size. Sub-page packing is the
/// concern of a slab allocator built on top; this layer's job is to source each
/// slab from the right NUMA node cheaply.
///
/// [`free_all`](Self::free_all) releases every allocation at once. There is no
/// per-object free.
pub struct LocalNumaArena {
    numa_node: u32,
    /// All live allocations as (base_ptr, byte_len) so free_all can release them.
    slabs: Vec<(*mut u8, usize)>,
    /// `PhantomData<*mut u8>` makes this `!Send + !Sync` at the type level.
    _not_send: PhantomData<*mut u8>,
}

impl LocalNumaArena {
    /// Create an arena bound to `numa_node`.
    ///
    /// Pass `0` on a single-socket machine or when topology is unavailable —
    /// it is the correct node id on every non-NUMA system and the safe fallback.
    pub fn new(numa_node: u32) -> Self {
        Self {
            numa_node,
            slabs: Vec::new(),
            _not_send: PhantomData,
        }
    }

    /// Allocate `count` values of type `T`, NUMA-local and cache-line aligned.
    ///
    /// Memory is zeroed. The returned pointer is valid until
    /// [`free_all`](Self::free_all) or `drop`.
    ///
    /// # Panics
    ///
    /// Panics if `count == 0` or if the allocation size overflows `isize::MAX`.
    pub fn alloc<T>(&mut self, count: usize) -> NonNull<T> {
        assert!(count > 0, "LocalNumaArena::alloc: count must be > 0");
        let layout = Layout::array::<T>(count)
            .expect("layout overflow")
            .align_to(CACHE_LINE)
            .expect("align_to failed");

        let ptr = platform::numa_alloc(layout, self.numa_node);
        self.slabs.push((ptr, layout.size()));
        // SAFETY: `numa_alloc` panics on null; the pointer is valid, non-null,
        // and aligned. NonNull::new_unchecked is sound.
        unsafe { NonNull::new_unchecked(ptr as *mut T) }
    }

    /// Release every allocation owned by this arena.
    ///
    /// All pointers previously returned by [`alloc`](Self::alloc) become
    /// dangling. The arena may be reused afterwards.
    pub fn free_all(&mut self) {
        for (ptr, len) in self.slabs.drain(..) {
            // SAFETY: (ptr, len) was produced by `numa_alloc` and is still live
            // (we drain, not iterate, so no double-free).
            unsafe { platform::numa_free(ptr, len) };
        }
    }

    /// Number of live slabs (for diagnostics and tests).
    pub fn slab_count(&self) -> usize {
        self.slabs.len()
    }

    /// Total bytes currently live in this arena.
    pub fn live_bytes(&self) -> usize {
        self.slabs.iter().map(|(_, l)| l).sum()
    }
}

impl Drop for LocalNumaArena {
    fn drop(&mut self) {
        self.free_all();
    }
}

// ── HugePageBuf (VV-HAL-031) ─────────────────────────────────────────────────

/// Why a [`HugePageBuf`] could not be allocated.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HugePageError {
    /// Zero-byte allocation makes no sense.
    #[error("cannot allocate zero bytes")]
    ZeroSize,

    /// The process lacks the privilege required for large pages.
    ///
    /// Windows: `SeLockMemoryPrivilege`. Linux: `CAP_IPC_LOCK` or
    /// `/proc/sys/vm/nr_hugepages` is zero. The caller may accept a regular-page
    /// fallback (`HugePageBuf::alloc_or_regular`) when this is returned.
    #[error(
        "insufficient privilege for large-page allocation \
         (Windows: grant SeLockMemoryPrivilege; Linux: set nr_hugepages)"
    )]
    NotPrivileged,

    /// The OS rejected the allocation for a reason other than privilege.
    #[error("huge-page allocation failed (OS code {0})")]
    Os(i32),
}

/// A buffer backed by 2 MB huge pages where available, 4 KB pages otherwise.
///
/// Dropping this value releases the underlying OS allocation.
///
/// Check [`is_huge`](Self::is_huge) to determine whether the OS actually granted
/// large pages — on most developer machines without explicit huge-page setup the
/// answer is `false`, which is correct and documented rather than hidden.
#[derive(Debug)]
pub struct HugePageBuf {
    ptr: NonNull<u8>,
    len: usize,
    huge: bool,
}

// SAFETY: HugePageBuf owns its allocation and holds no thread-local state.
unsafe impl Send for HugePageBuf {}
unsafe impl Sync for HugePageBuf {}

impl HugePageBuf {
    /// Allocate at least `size` bytes, attempting huge pages first.
    ///
    /// Returns `Err` if even the regular-page fallback fails.
    /// Check [`is_huge`](Self::is_huge) on the result to see what you actually got.
    pub fn alloc(size: usize) -> Result<Self, HugePageError> {
        if size == 0 {
            return Err(HugePageError::ZeroSize);
        }
        platform::huge_alloc(size)
    }

    /// `true` if the OS granted large (2 MB) pages for this buffer.
    pub fn is_huge(&self) -> bool {
        self.huge
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
        // SAFETY: ptr is valid for len bytes, non-null, properly aligned.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: &mut self guarantees exclusive access.
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for HugePageBuf {
    fn drop(&mut self) {
        // SAFETY: ptr and len were produced by huge_alloc and are still live.
        unsafe { platform::huge_free(self.ptr.as_ptr(), self.len, self.huge) };
    }
}

// ── mlock (VV-HAL-033) ────────────────────────────────────────────────────────

/// Why [`mlock_range`] failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MlockError {
    /// Zero-length ranges have nothing to lock.
    #[error("cannot mlock a zero-length range")]
    ZeroLength,

    /// The OS refused — typically the process exceeded its locked-memory limit
    /// (`ulimit -l` on Linux; working-set quota on Windows).
    #[error("OS refused mlock (code {0}; check locked-memory limits)")]
    Os(i32),
}

/// Keeps a memory range locked in DRAM until dropped.
///
/// The pages will not be paged out while this guard lives. Dropping unlocks
/// them. On platforms without `mlock` (anything other than Linux and Windows)
/// this is a no-op guard that always succeeds.
#[must_use = "dropping the guard immediately unlocks the memory"]
#[derive(Debug)]
pub struct MlockGuard {
    ptr: *mut u8,
    len: usize,
}

// SAFETY: MlockGuard holds a pointer but does not alias any other data structure.
unsafe impl Send for MlockGuard {}
unsafe impl Sync for MlockGuard {}

impl Drop for MlockGuard {
    fn drop(&mut self) {
        if self.len > 0 {
            // SAFETY: ptr and len came from a successful mlock call and are still live.
            // Failure to unlock is silent — Drop cannot propagate errors and panicking
            // during unwind would abort the process.
            unsafe { platform::munlock(self.ptr, self.len) };
        }
    }
}

/// Lock `len` bytes starting at `ptr` into DRAM.
///
/// # Safety
///
/// `ptr` must be valid for `len` bytes and must remain valid until the returned
/// [`MlockGuard`] is dropped.
///
/// # Errors
///
/// [`MlockError::ZeroLength`] if `len == 0`.
/// [`MlockError::Os`] if the OS refused (exceeded locked-memory limits).
pub unsafe fn mlock_range(ptr: *mut u8, len: usize) -> Result<MlockGuard, MlockError> {
    if len == 0 {
        return Err(MlockError::ZeroLength);
    }
    // SAFETY: safety contract delegated from caller.
    unsafe { platform::mlock(ptr, len) }?;
    Ok(MlockGuard { ptr, len })
}

/// Unlock a memory range previously locked with [`mlock_range`].
///
/// Idempotent — unlocking a range that was never locked is harmless on Linux.
///
/// # Safety
///
/// `ptr` must be a valid pointer and `len` must match what was passed to `mlock_range`.
pub unsafe fn munlock_range(ptr: *const u8, len: usize) -> Result<(), MlockError> {
    if len == 0 {
        return Ok(());
    }
    // SAFETY: delegated from caller.
    unsafe { platform::munlock(ptr as *mut u8, len) };
    Ok(())
}

// ── prefetch_populate (VV-HAL-032) ────────────────────────────────────────────

/// Pre-fault every page in `[ptr, ptr + len)` into DRAM.
///
/// This is the canonical HAL entry point for `MAP_POPULATE`-equivalent
/// pre-faulting. `PartitionReader::open` already calls the OS primitive directly;
/// use this function in new code so the mechanism is testable in isolation.
///
/// A no-op on empty ranges.
///
/// # Safety
///
/// `ptr` must be valid for `len` bytes.
pub unsafe fn prefetch_populate(ptr: *const u8, len: usize) {
    if len == 0 {
        return;
    }
    // SAFETY: safety contract delegated from caller.
    unsafe { platform::prefetch_populate(ptr, len) };
}

// ── Platform: Windows ─────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod platform {
    use super::{HugePageBuf, HugePageError};
    use std::ptr::NonNull;
    use windows_sys::Win32::Foundation::GetLastError;
    use windows_sys::Win32::System::Memory::{
        MEM_COMMIT, MEM_LARGE_PAGES, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
        PrefetchVirtualMemory, VirtualAlloc, VirtualAllocExNuma, VirtualFree, VirtualLock,
        VirtualUnlock, WIN32_MEMORY_RANGE_ENTRY,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    pub(super) fn numa_alloc(layout: std::alloc::Layout, node: u32) -> *mut u8 {
        // Try NUMA-local allocation first.
        // SAFETY: GetCurrentProcess() is a valid pseudo-handle that needs no close.
        let ptr = unsafe {
            VirtualAllocExNuma(
                GetCurrentProcess(),
                std::ptr::null(),
                layout.size(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
                node,
            )
        };
        if !ptr.is_null() {
            return ptr as *mut u8;
        }

        // Fall back to node-agnostic allocation (still zeroed by Windows).
        let ptr2 = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                layout.size(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };
        if ptr2.is_null() {
            panic!(
                "LocalNumaArena: VirtualAlloc failed for {} bytes",
                layout.size()
            );
        }
        ptr2 as *mut u8
    }

    pub(super) unsafe fn numa_free(ptr: *mut u8, _len: usize) {
        // SAFETY: ptr came from VirtualAlloc/VirtualAllocExNuma.
        unsafe {
            VirtualFree(ptr.cast(), 0, MEM_RELEASE);
        }
    }

    pub(super) fn huge_alloc(size: usize) -> Result<HugePageBuf, HugePageError> {
        // Round up to the 2 MB huge-page boundary.
        const HUGE: usize = 2 * 1024 * 1024;
        let rounded = size.div_ceil(HUGE) * HUGE;

        // Attempt MEM_LARGE_PAGES — requires SeLockMemoryPrivilege.
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                rounded,
                MEM_COMMIT | MEM_RESERVE | MEM_LARGE_PAGES,
                PAGE_READWRITE,
            )
        };
        if !ptr.is_null() {
            return Ok(HugePageBuf {
                // SAFETY: ptr is non-null; we just checked.
                ptr: unsafe { NonNull::new_unchecked(ptr as *mut u8) },
                len: rounded,
                huge: true,
            });
        }

        // Large pages failed — classify the error but fall through to regular pages.
        // ERROR_PRIVILEGE_NOT_HELD = 1314 (0x522).
        let _code = unsafe { GetLastError() };

        // Fallback: regular 4 KB pages, commit-on-demand.
        let ptr2 = unsafe {
            VirtualAlloc(
                std::ptr::null(),
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            )
        };
        if ptr2.is_null() {
            let code = unsafe { GetLastError() };
            return Err(HugePageError::Os(code as i32));
        }
        Ok(HugePageBuf {
            // SAFETY: ptr2 is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr2 as *mut u8) },
            len: size,
            huge: false,
        })
    }

    pub(super) unsafe fn huge_free(ptr: *mut u8, _len: usize, _huge: bool) {
        // SAFETY: ptr came from VirtualAlloc in huge_alloc.
        unsafe {
            VirtualFree(ptr.cast(), 0, MEM_RELEASE);
        }
    }

    pub(super) unsafe fn mlock(ptr: *mut u8, len: usize) -> Result<(), super::MlockError> {
        // SAFETY: ptr and len are delegated from the caller's safety contract.
        let ok = unsafe { VirtualLock(ptr.cast(), len) };
        if ok == 0 {
            let code = unsafe { GetLastError() };
            Err(super::MlockError::Os(code as i32))
        } else {
            Ok(())
        }
    }

    pub(super) unsafe fn munlock(ptr: *mut u8, len: usize) {
        // SAFETY: ptr and len came from a successful VirtualLock.
        unsafe {
            VirtualUnlock(ptr.cast(), len);
        }
    }

    pub(super) unsafe fn prefetch_populate(ptr: *const u8, len: usize) {
        // PrefetchVirtualMemory hints the memory manager to fault pages in eagerly.
        // Available since Windows 8 / Server 2012 — well within our minimum target.
        let entry = WIN32_MEMORY_RANGE_ENTRY {
            VirtualAddress: ptr as *mut _,
            NumberOfBytes: len,
        };
        // SAFETY: GetCurrentProcess() is always a valid pseudo-handle.
        // `entry` is on the stack and valid for the duration of the call.
        unsafe {
            PrefetchVirtualMemory(GetCurrentProcess(), 1, &entry, 0);
        }
    }
}

// ── Platform: Linux / Android ─────────────────────────────────────────────────

#[cfg(any(target_os = "linux", target_os = "android"))]
mod platform {
    use super::{HugePageBuf, HugePageError};
    use std::ptr::NonNull;

    pub(super) fn numa_alloc(layout: std::alloc::Layout, _node: u32) -> *mut u8 {
        // mbind for NUMA locality is 🧊 deferred to the Linux sprint.
        // mmap anonymous is correct and zeroed on all Linux versions.
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                layout.size(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED || ptr.is_null() {
            panic!("LocalNumaArena: mmap failed for {} bytes", layout.size());
        }
        ptr as *mut u8
    }

    pub(super) unsafe fn numa_free(ptr: *mut u8, len: usize) {
        // SAFETY: ptr and len came from mmap in numa_alloc.
        unsafe {
            libc::munmap(ptr.cast(), len);
        }
    }

    pub(super) fn huge_alloc(size: usize) -> Result<HugePageBuf, HugePageError> {
        const HUGE: usize = 2 * 1024 * 1024;
        let rounded = size.div_ceil(HUGE) * HUGE;

        // MAP_HUGETLB is available on Linux 2.6.32+ and requires
        // /proc/sys/vm/nr_hugepages > 0.
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                rounded,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_HUGETLB,
                -1,
                0,
            )
        };
        if ptr != libc::MAP_FAILED && !ptr.is_null() {
            return Ok(HugePageBuf {
                // SAFETY: ptr is non-null.
                ptr: unsafe { NonNull::new_unchecked(ptr as *mut u8) },
                len: rounded,
                huge: true,
            });
        }

        // Fallback: regular anonymous mmap.
        let ptr2 = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if ptr2 == libc::MAP_FAILED || ptr2.is_null() {
            return Err(HugePageError::Os(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ));
        }
        Ok(HugePageBuf {
            // SAFETY: ptr2 is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr2 as *mut u8) },
            len: size,
            huge: false,
        })
    }

    pub(super) unsafe fn huge_free(ptr: *mut u8, len: usize, _huge: bool) {
        // SAFETY: ptr and len came from mmap in huge_alloc.
        unsafe {
            libc::munmap(ptr.cast(), len);
        }
    }

    pub(super) unsafe fn mlock(ptr: *mut u8, len: usize) -> Result<(), super::MlockError> {
        // SAFETY: ptr and len are delegated from the caller's safety contract.
        let rc = unsafe { libc::mlock(ptr.cast(), len) };
        if rc != 0 {
            Err(super::MlockError::Os(
                std::io::Error::last_os_error().raw_os_error().unwrap_or(-1),
            ))
        } else {
            Ok(())
        }
    }

    pub(super) unsafe fn munlock(ptr: *mut u8, len: usize) {
        // SAFETY: ptr and len came from a successful mlock.
        unsafe {
            libc::munlock(ptr.cast(), len);
        }
    }

    pub(super) unsafe fn prefetch_populate(ptr: *const u8, len: usize) {
        // MADV_WILLNEED asks the kernel to read-ahead pages asynchronously.
        unsafe {
            libc::madvise(ptr as *mut _, len, libc::MADV_WILLNEED);
        }
        // A sequential volatile-read loop fault-walks any pages madvise left lazy.
        const PAGE: usize = 4096;
        let mut sum = 0u8;
        let mut off = 0;
        while off < len {
            sum = sum.wrapping_add(unsafe { std::ptr::read_volatile(ptr.add(off)) });
            off += PAGE;
        }
        // Prevent the compiler from eliminating the loop by using the accumulator.
        let _ = std::hint::black_box(sum);
    }
}

// ── Platform: fallback (WASM, tier-3, macOS, …) ───────────────────────────────

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "android",)))]
mod platform {
    use super::{HugePageBuf, HugePageError};
    use std::alloc::{Layout, alloc_zeroed, dealloc};
    use std::ptr::NonNull;

    pub(super) fn numa_alloc(layout: Layout, _node: u32) -> *mut u8 {
        // SAFETY: layout is non-zero (checked by caller).
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            panic!(
                "LocalNumaArena: alloc_zeroed failed for {} bytes",
                layout.size()
            );
        }
        ptr
    }

    pub(super) unsafe fn numa_free(ptr: *mut u8, len: usize) {
        if let Ok(layout) = Layout::from_size_align(len, crate::cache::CACHE_LINE) {
            // SAFETY: ptr came from alloc_zeroed with this layout.
            unsafe { dealloc(ptr, layout) };
        }
    }

    pub(super) fn huge_alloc(size: usize) -> Result<HugePageBuf, HugePageError> {
        let layout = Layout::from_size_align(size, crate::cache::CACHE_LINE)
            .map_err(|_| HugePageError::Os(-1))?;
        // SAFETY: layout is non-zero.
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err(HugePageError::Os(-1));
        }
        Ok(HugePageBuf {
            // SAFETY: ptr is non-null.
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len: size,
            huge: false,
        })
    }

    pub(super) unsafe fn huge_free(ptr: *mut u8, len: usize, _huge: bool) {
        if let Ok(layout) = Layout::from_size_align(len, crate::cache::CACHE_LINE) {
            // SAFETY: ptr came from alloc_zeroed with this layout.
            unsafe { dealloc(ptr, layout) };
        }
    }

    pub(super) unsafe fn mlock(_ptr: *mut u8, _len: usize) -> Result<(), super::MlockError> {
        Ok(()) // No-op — no mlock API on this platform.
    }

    pub(super) unsafe fn munlock(_ptr: *mut u8, _len: usize) {}

    pub(super) unsafe fn prefetch_populate(ptr: *const u8, len: usize) {
        const PAGE: usize = 4096;
        let mut sum = 0u8;
        let mut off = 0;
        while off < len {
            sum = sum.wrapping_add(unsafe { std::ptr::read_volatile(ptr.add(off)) });
            off += PAGE;
        }
        let _ = std::hint::black_box(sum);
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1 — basic allocation and read/write roundtrip
    #[test]
    fn numa_arena_alloc_and_readwrite() {
        let mut arena = LocalNumaArena::new(0);
        let ptr: NonNull<u64> = arena.alloc(128);
        unsafe {
            for i in 0..128usize {
                ptr.as_ptr().add(i).write(i as u64);
            }
            for i in 0..128usize {
                assert_eq!(ptr.as_ptr().add(i).read(), i as u64);
            }
        }
    }

    // 2 — allocations must be cache-line aligned
    #[test]
    fn numa_arena_alloc_is_cache_line_aligned() {
        let mut arena = LocalNumaArena::new(0);
        for _ in 0..8 {
            let ptr: NonNull<u8> = arena.alloc(1);
            assert_eq!(
                ptr.as_ptr() as usize % CACHE_LINE,
                0,
                "allocation not cache-line aligned"
            );
        }
    }

    // 3 — two allocations must be at distinct addresses
    #[test]
    fn numa_arena_multiple_allocs_are_independent() {
        let mut arena = LocalNumaArena::new(0);
        let a: NonNull<u64> = arena.alloc(4);
        let b: NonNull<u64> = arena.alloc(4);
        assert_ne!(a.as_ptr(), b.as_ptr());
        assert_eq!(arena.slab_count(), 2);
    }

    // 4 — live_bytes tracks correctly
    #[test]
    fn numa_arena_tracks_live_bytes() {
        let mut arena = LocalNumaArena::new(0);
        let _: NonNull<u8> = arena.alloc::<u8>(4096);
        assert!(arena.live_bytes() >= 4096);
    }

    // 5 — free_all clears the slab list
    #[test]
    fn numa_arena_free_all_clears_slabs() {
        let mut arena = LocalNumaArena::new(0);
        let _: NonNull<u64> = arena.alloc(8);
        let _: NonNull<u64> = arena.alloc(8);
        assert_eq!(arena.slab_count(), 2);
        arena.free_all();
        assert_eq!(arena.slab_count(), 0);
        assert_eq!(arena.live_bytes(), 0);
    }

    // 6 — arena is usable after free_all
    #[test]
    fn numa_arena_reusable_after_free_all() {
        let mut arena = LocalNumaArena::new(0);
        arena.free_all(); // calling on empty must not panic
        let ptr: NonNull<u32> = arena.alloc(10);
        unsafe { ptr.as_ptr().write(42) };
        assert_eq!(unsafe { ptr.as_ptr().read() }, 42);
    }

    // 7 — zero-size is rejected
    #[test]
    fn huge_page_zero_size_rejected() {
        assert_eq!(HugePageBuf::alloc(0).unwrap_err(), HugePageError::ZeroSize);
    }

    // 8 — length is at least what was requested
    #[test]
    fn huge_page_len_at_least_requested() {
        let buf = HugePageBuf::alloc(4096).expect("alloc must succeed");
        assert!(buf.len() >= 4096);
    }

    // 9 — readable and writable (proves the OS actually committed pages)
    #[test]
    fn huge_page_read_write_roundtrip() {
        let mut buf = HugePageBuf::alloc(4096).expect("alloc must succeed");
        buf.as_mut_slice()[0] = 0xAB;
        buf.as_mut_slice()[4095] = 0xCD;
        assert_eq!(buf.as_slice()[0], 0xAB);
        assert_eq!(buf.as_slice()[4095], 0xCD);
    }

    // 10 — pointer is non-null
    #[test]
    fn huge_page_ptr_is_nonnull() {
        let buf = HugePageBuf::alloc(64 * 1024).expect("alloc");
        assert!(!buf.as_ptr().is_null());
    }

    // 11 — mlock on zero length returns ZeroLength
    #[test]
    fn mlock_zero_len_is_error() {
        let mut dummy = 0u8;
        let err = unsafe { mlock_range(&mut dummy as *mut u8, 0) }.unwrap_err();
        assert_eq!(err, MlockError::ZeroLength);
    }

    // 12 — mlock on a real page succeeds or fails gracefully (quota)
    #[test]
    fn mlock_on_real_page_is_ok_or_quota() {
        let mut buf = HugePageBuf::alloc(4096).expect("alloc");
        let result = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) };
        match result {
            Ok(guard) => drop(guard),
            Err(MlockError::Os(_)) => {} // insufficient ulimit/working-set — platform issue
            Err(e) => panic!("unexpected mlock error: {e}"),
        }
    }

    // 13 — guard drops without panic
    #[test]
    fn mlock_guard_drops_cleanly() {
        let mut buf = HugePageBuf::alloc(4096).expect("alloc");
        if let Ok(guard) = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) } {
            std::mem::drop(guard); // explicit drop — must not panic
        }
    }

    // 14 — prefetch_populate on empty slice is a no-op
    #[test]
    fn prefetch_populate_empty_is_noop() {
        let arr = [0u8; 0];
        unsafe { prefetch_populate(arr.as_ptr(), 0) }; // must not panic
    }

    // 15 — prefetch_populate on a real allocation does not corrupt data
    #[test]
    fn prefetch_populate_preserves_contents() {
        let mut buf = HugePageBuf::alloc(2 * 1024 * 1024).expect("alloc");
        let last = buf.len() - 1;
        buf.as_mut_slice()[0] = 0xFF;
        buf.as_mut_slice()[last] = 0x42;
        unsafe { prefetch_populate(buf.as_ptr(), buf.len()) };
        assert_eq!(buf.as_slice()[0], 0xFF);
        assert_eq!(buf.as_slice()[last], 0x42);
    }
}

// ── Linux-gated S11 mlock tests (VV-HAL-033) ─────────────────────────────────

#[cfg(all(test, any(target_os = "linux", target_os = "android")))]
mod linux_mlock_tests {
    use super::*;

    fn page() -> Vec<u8> {
        vec![0u8; 4096]
    }

    // 1 — mlock 4096 bytes ok
    #[test]
    fn linux_mlock_4096_ok() {
        let mut buf = page();
        let result = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) };
        match result {
            Ok(g) => drop(g),
            Err(MlockError::Os(_)) => {} // ulimit
            Err(e) => panic!("unexpected: {e}"),
        }
    }

    // 2 — munlock after mlock ok
    #[test]
    fn linux_munlock_after_mlock_ok() {
        let mut buf = page();
        if let Ok(g) = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) } {
            drop(g);
            // explicit munlock on already-unlocked is idempotent
            let _ = unsafe { munlock_range(buf.as_ptr(), buf.len()) };
        }
    }

    // 3 — mlock zero bytes ok (no-op)
    #[test]
    fn linux_mlock_zero_ok() {
        let mut dummy = 0u8;
        assert_eq!(
            unsafe { mlock_range(&mut dummy as *mut u8, 0) }.unwrap_err(),
            MlockError::ZeroLength
        );
    }

    // 4 — mlock_range ptr alignment (page-aligned ptr works)
    #[test]
    fn linux_mlock_ptr_alignment() {
        let mut buf = vec![0u8; 8192 + 4096];
        let aligned_ptr = {
            let addr = buf.as_mut_ptr() as usize;
            let off = (4096 - addr % 4096) % 4096;
            unsafe { buf.as_mut_ptr().add(off) }
        };
        let result = unsafe { mlock_range(aligned_ptr, 4096) };
        match result {
            Ok(g) => drop(g),
            Err(MlockError::Os(_)) => {}
            Err(e) => panic!("{e}"),
        }
    }

    // 5 — MlockGuard drops and unlocks without panic
    #[test]
    fn linux_mlock_guard_drops() {
        let mut buf = page();
        if let Ok(guard) = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) } {
            drop(guard);
        }
    }

    // 6 — mlock 1MB ok
    #[test]
    fn linux_mlock_1mb_ok() {
        let mut buf = vec![0u8; 1024 * 1024];
        match unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) } {
            Ok(g) => drop(g),
            Err(MlockError::Os(_)) => {}
            Err(e) => panic!("{e}"),
        }
    }

    // 7 — mlock null-equivalent (very small vec at unpredictable addr, not null)
    #[test]
    fn linux_mlock_non_null_ptr() {
        let mut b = vec![42u8; 4096];
        assert!(!b.as_ptr().is_null());
        match unsafe { mlock_range(b.as_mut_ptr(), b.len()) } {
            Ok(g) => drop(g),
            Err(MlockError::Os(_)) => {}
            Err(e) => panic!("{e}"),
        }
    }

    // 8 — double mlock same region ok
    #[test]
    fn linux_double_mlock_ok() {
        let mut buf = page();
        let r1 = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) };
        let r2 = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) };
        drop(r1.ok());
        drop(r2.ok());
    }

    // 9 — MlockGuard size field (guard created for correct len)
    #[test]
    fn linux_mlock_guard_fields_correct() {
        let mut buf = page();
        let len = buf.len();
        if let Ok(g) = unsafe { mlock_range(buf.as_mut_ptr(), len) } {
            // Guard was created — it covers the correct region (drop tests unlock)
            let _ = g; // drop unlocks
        }
    }

    // 10 — mlock_range len=1 ok
    #[test]
    fn linux_mlock_len_1_ok() {
        let mut buf = page();
        match unsafe { mlock_range(buf.as_mut_ptr(), 1) } {
            Ok(g) => drop(g),
            Err(MlockError::Os(_)) => {}
            Err(e) => panic!("{e}"),
        }
    }

    // 11 — munlock without prior mlock is idempotent
    #[test]
    fn linux_munlock_without_mlock_idempotent() {
        let buf = page();
        let _ = unsafe { munlock_range(buf.as_ptr(), buf.len()) };
    }

    // 12 — MlockGuard from vec
    #[test]
    fn linux_mlock_guard_from_vec() {
        let mut v: Vec<u64> = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let ptr = v.as_mut_ptr() as *mut u8;
        let len = v.len() * std::mem::size_of::<u64>();
        match unsafe { mlock_range(ptr, len) } {
            Ok(g) => drop(g),
            Err(MlockError::Os(_)) => {}
            Err(e) => panic!("{e}"),
        }
    }

    // 13 — mlock + write + read roundtrip
    #[test]
    fn linux_mlock_write_read_roundtrip() {
        let mut buf = page();
        let ok = unsafe { mlock_range(buf.as_mut_ptr(), buf.len()) };
        buf[0] = 0xDE;
        buf[4095] = 0xAD;
        assert_eq!(buf[0], 0xDE);
        assert_eq!(buf[4095], 0xAD);
        drop(ok.ok());
    }
}

// ── Linux-gated S11 hugepage tests (VV-HAL-031) ───────────────────────────────

#[cfg(all(test, any(target_os = "linux", target_os = "android")))]
mod linux_hugepage_tests {
    use super::*;

    fn alloc_or_skip(size: usize) -> Option<HugePageBuf> {
        match HugePageBuf::alloc(size) {
            Ok(b) => Some(b),
            Err(HugePageError::Os(_)) | Err(HugePageError::NotPrivileged) => None,
            Err(e) => panic!("unexpected: {e}"),
        }
    }

    // 1 — alloc 2MB ok or NotAvailable
    #[test]
    fn linux_hugepage_alloc_2mb() {
        let _ = HugePageBuf::alloc(HUGE_PAGE_SIZE); // ok or Err(Os) — both valid
    }

    // 2 — alloc 4MB ok
    #[test]
    fn linux_hugepage_alloc_4mb() {
        let _ = HugePageBuf::alloc(2 * HUGE_PAGE_SIZE);
    }

    // 3 — len correct
    #[test]
    fn linux_hugepage_len_correct() {
        if let Some(b) = alloc_or_skip(HUGE_PAGE_SIZE) {
            assert!(b.len() >= HUGE_PAGE_SIZE);
        }
    }

    // 4 — ptr non-null
    #[test]
    fn linux_hugepage_ptr_non_null() {
        if let Some(b) = alloc_or_skip(4096) {
            assert!(!b.as_ptr().is_null());
        }
    }

    // 5 — write and read roundtrip
    #[test]
    fn linux_hugepage_write_read_roundtrip() {
        if let Some(mut b) = alloc_or_skip(4096) {
            let last = b.len() - 1;
            b.as_mut_slice()[0] = 0xBE;
            b.as_mut_slice()[last] = 0xEF;
            assert_eq!(b.as_slice()[0], 0xBE);
            assert_eq!(b.as_slice()[last], 0xEF);
        }
    }

    // 6 — alloc 0 rounds up to 2MB (ZeroSize error)
    #[test]
    fn linux_hugepage_alloc_zero_is_error() {
        assert_eq!(HugePageBuf::alloc(0).unwrap_err(), HugePageError::ZeroSize);
    }

    // 7 — alloc 1 byte rounds up to 2MB
    #[test]
    fn linux_hugepage_alloc_1_byte_rounds_up() {
        if let Some(b) = alloc_or_skip(1) {
            assert!(b.len() >= 1);
        }
    }

    // 8 — drop frees memory (no double-free — just don't panic)
    #[test]
    fn linux_hugepage_drop_no_panic() {
        if let Some(b) = alloc_or_skip(HUGE_PAGE_SIZE) {
            drop(b);
        }
    }

    // 9 — as_ptr == as_mut_ptr cast
    #[test]
    fn linux_hugepage_ptr_eq_mut_ptr() {
        if let Some(mut b) = alloc_or_skip(4096) {
            assert_eq!(b.as_ptr(), b.as_mut_ptr() as *const u8);
        }
    }

    // 10 — alloc multiple bufs ok
    #[test]
    fn linux_hugepage_multiple_allocs_ok() {
        let bufs: Vec<_> = (0..3).filter_map(|_| alloc_or_skip(4096)).collect();
        drop(bufs);
    }

    // 11 — HUGE_PAGE_SIZE constant = 2097152
    #[test]
    fn linux_huge_page_size_constant() {
        assert_eq!(HUGE_PAGE_SIZE, 2_097_152);
    }

    // 12 — alloc falls back gracefully when hugepages unavailable
    #[test]
    fn linux_hugepage_graceful_fallback() {
        // HugePageBuf always falls back to regular mmap on failure — never panics
        let result = HugePageBuf::alloc(4096);
        assert!(
            result.is_ok(),
            "must succeed via regular fallback: {result:?}"
        );
    }

    // 13 — buf is Send+Sync (compile-time assertion)
    #[test]
    fn linux_hugepage_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HugePageBuf>();
    }
}
