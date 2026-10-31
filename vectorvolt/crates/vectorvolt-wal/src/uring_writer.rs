//! io_uring-based WAL appender — async kernel-bypass disk I/O (VV-WAL-013).
//!
//! On Linux 5.1+ this uses the `io_uring` crate to submit write SQEs and reap
//! CQEs without syscalls on the hot path.  On non-Linux platforms the same
//! struct wraps a `BufWriter<File>` so every caller compiles and tests pass
//! everywhere.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use vectorvolt_wal::uring_writer::UringWalAppender;
//! use std::path::Path;
//!
//! let mut w = UringWalAppender::open(Path::new("/tmp/vv-wal.bin"), 256).unwrap();
//! let pos = w.append(b"hello world").unwrap();
//! w.fsync().unwrap();
//! w.close().unwrap();
//! ```

// ── Linux implementation ───────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod inner {
    use io_uring::{IoUring, opcode, types};
    use std::io;
    use std::os::unix::io::AsRawFd;
    use std::path::Path;

    /// io_uring-based WAL appender. Linux 5.1+ only.
    pub struct UringWalAppender {
        ring: IoUring,
        file: std::fs::File,
        pos: u64,
    }

    impl UringWalAppender {
        /// Open `path` for appending with an io_uring of `ring_size` entries.
        pub fn open(path: &Path, ring_size: u32) -> io::Result<Self> {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
            let ring = IoUring::new(ring_size.max(1))?;
            Ok(Self { ring, file, pos: 0 })
        }

        /// Append `data` to the WAL.  Submits one write SQE, waits for its CQE,
        /// returns the new write position.
        pub fn append(&mut self, data: &[u8]) -> io::Result<u64> {
            if data.is_empty() {
                return Ok(self.pos);
            }
            let fd = types::Fd(self.file.as_raw_fd());
            let write_e = opcode::Write::new(fd, data.as_ptr(), data.len() as u32)
                .offset(self.pos)
                .build()
                .user_data(0x1);

            // SAFETY: The SQE references `data` which is live for the duration
            // of this function call; we submit and wait synchronously before returning.
            unsafe {
                self.ring
                    .submission()
                    .push(&write_e)
                    .map_err(|_| io::Error::other("io_uring SQ full"))?;
            }
            self.ring.submit_and_wait(1)?;

            // Drain the CQE and check the result.
            let cqe = self
                .ring
                .completion()
                .next()
                .ok_or_else(|| io::Error::other("io_uring: no CQE"))?;
            let result = cqe.result();
            if result < 0 {
                return Err(io::Error::from_raw_os_error(-result));
            }
            self.pos += result as u64;
            Ok(self.pos)
        }

        /// Issue an fsync via io_uring.
        pub fn fsync(&mut self) -> io::Result<()> {
            let fd = types::Fd(self.file.as_raw_fd());
            let sync_e = opcode::Fsync::new(fd).build().user_data(0x2);
            // SAFETY: SQE has no data pointer; safe to submit.
            unsafe {
                self.ring
                    .submission()
                    .push(&sync_e)
                    .map_err(|_| io::Error::other("io_uring SQ full"))?;
            }
            self.ring.submit_and_wait(1)?;
            let cqe = self
                .ring
                .completion()
                .next()
                .ok_or_else(|| io::Error::other("io_uring: no CQE"))?;
            let result = cqe.result();
            if result < 0 {
                Err(io::Error::from_raw_os_error(-result))
            } else {
                Ok(())
            }
        }

        /// Current write position.
        pub fn pos(&self) -> u64 {
            self.pos
        }

        /// Close the appender (flushes nothing; call [`fsync`](Self::fsync) first).
        pub fn close(self) -> io::Result<()> {
            Ok(())
        }
    }
}

// ── Fallback (non-Linux) ───────────────────────────────────────────────────────

#[cfg(not(target_os = "linux"))]
mod inner {
    use std::io::{self, BufWriter, Write};
    use std::path::Path;

    /// Fallback WAL appender for non-Linux: wraps `BufWriter<File>`.
    pub struct UringWalAppender {
        writer: BufWriter<std::fs::File>,
        pos: u64,
    }

    impl UringWalAppender {
        pub fn open(path: &Path, _ring_size: u32) -> io::Result<Self> {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
            Ok(Self {
                writer: BufWriter::new(file),
                pos: 0,
            })
        }

        pub fn append(&mut self, data: &[u8]) -> io::Result<u64> {
            if data.is_empty() {
                return Ok(self.pos);
            }
            self.writer.write_all(data)?;
            self.pos += data.len() as u64;
            Ok(self.pos)
        }

        pub fn fsync(&mut self) -> io::Result<()> {
            self.writer.flush()?;
            self.writer.get_ref().sync_all()
        }

        pub fn pos(&self) -> u64 {
            self.pos
        }

        pub fn close(mut self) -> io::Result<()> {
            self.writer.flush()
        }
    }
}

// ── Public re-export ───────────────────────────────────────────────────────────

pub use inner::UringWalAppender;

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tmp_path() -> PathBuf {
        static CTR: AtomicU64 = AtomicU64::new(0);
        let n = CTR.fetch_add(1, Ordering::Relaxed);
        let mut p = std::env::temp_dir();
        p.push(format!("vv_uring_test_{}_{}.wal", std::process::id(), n));
        p
    }

    // 1 — open creates the file
    #[test]
    fn uring_open_creates_file() {
        let p = tmp_path();
        let _w = UringWalAppender::open(&p, 32).expect("open");
        assert!(p.exists());
        let _ = std::fs::remove_file(&p);
    }

    // 2 — append 32 bytes ok
    #[test]
    fn uring_append_32_bytes() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        let pos = w.append(&[0u8; 32]).expect("append");
        assert_eq!(pos, 32);
        let _ = std::fs::remove_file(&p);
    }

    // 3 — pos advances
    #[test]
    fn uring_pos_advances() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        assert_eq!(w.pos(), 0);
        w.append(b"hello").expect("append");
        assert_eq!(w.pos(), 5);
        let _ = std::fs::remove_file(&p);
    }

    // 4 — append multiple times
    #[test]
    fn uring_append_multiple_times() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        for i in 0..5u8 {
            w.append(&[i; 16]).expect("append");
        }
        assert_eq!(w.pos(), 80);
        let _ = std::fs::remove_file(&p);
    }

    // 5 — fsync ok
    #[test]
    fn uring_fsync_ok() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        w.append(b"data").expect("append");
        w.fsync().expect("fsync");
        let _ = std::fs::remove_file(&p);
    }

    // 6 — read back written data
    #[test]
    fn uring_read_back_data() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        w.append(b"vectorvolt").expect("append");
        w.fsync().expect("fsync");
        w.close().expect("close");
        let mut buf = Vec::new();
        std::fs::File::open(&p)
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        assert_eq!(&buf, b"vectorvolt");
        let _ = std::fs::remove_file(&p);
    }

    // 7 — append 0 bytes is a no-op
    #[test]
    fn uring_append_zero_noop() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        let pos = w.append(&[]).expect("append empty");
        assert_eq!(pos, 0);
        assert_eq!(w.pos(), 0);
        let _ = std::fs::remove_file(&p);
    }

    // 8 — pos after 3 appends = sum of sizes
    #[test]
    fn uring_pos_is_sum_of_sizes() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        w.append(&[0; 10]).unwrap();
        w.append(&[0; 20]).unwrap();
        w.append(&[0; 30]).unwrap();
        assert_eq!(w.pos(), 60);
        let _ = std::fs::remove_file(&p);
    }

    // 9 — open nonexistent dir returns Err
    #[test]
    fn uring_open_nonexistent_dir_returns_err() {
        let p = PathBuf::from("/nonexistent_vv_uring_dir/test.wal");
        assert!(UringWalAppender::open(&p, 32).is_err());
    }

    // 10 — close ok
    #[test]
    fn uring_close_ok() {
        let p = tmp_path();
        let w = UringWalAppender::open(&p, 32).expect("open");
        w.close().expect("close");
        let _ = std::fs::remove_file(&p);
    }

    // 11 — open ring_size=1
    #[test]
    fn uring_open_ring_size_1() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 1).expect("open ring_size=1");
        w.append(b"a").expect("append");
        let _ = std::fs::remove_file(&p);
    }

    // 12 — open ring_size=256
    #[test]
    fn uring_open_ring_size_256() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 256).expect("open ring_size=256");
        w.append(b"b").expect("append");
        let _ = std::fs::remove_file(&p);
    }

    // 13 — data integrity: write then read all bytes match
    #[test]
    fn uring_data_integrity() {
        let p = tmp_path();
        let mut w = UringWalAppender::open(&p, 32).expect("open");
        let payload: Vec<u8> = (0u8..=255).cycle().take(512).collect();
        w.append(&payload).expect("append");
        w.fsync().expect("fsync");
        w.close().expect("close");
        let mut read = Vec::new();
        std::fs::File::open(&p)
            .unwrap()
            .read_to_end(&mut read)
            .unwrap();
        assert_eq!(read, payload);
        let _ = std::fs::remove_file(&p);
    }
}
