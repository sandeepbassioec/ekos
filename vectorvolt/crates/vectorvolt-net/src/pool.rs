//! Async connection pool for VectorVolt (VV-NET-003).
//!
//! [`ConnectionPool`] maintains a fixed-size set of TCP connections.
//! Callers call [`ConnectionPool::acquire`] (with a 5-second timeout) and
//! get a [`PooledConn`] RAII guard; when dropped, the connection is returned.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;

use crate::connection::Connection;
use crate::{NetError, NetResult};

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// A fixed-size pool of [`Connection`]s to a single server address.
#[derive(Debug)]
pub struct ConnectionPool {
    addr: SocketAddr,
    max: usize,
    /// Semaphore: limits concurrent checkouts to `max`.
    sem: Arc<Semaphore>,
    /// Idle connections waiting to be acquired.
    idle: Arc<Mutex<Vec<Connection>>>,
}

impl ConnectionPool {
    /// Create a pool and eagerly open `max` connections to `addr`.
    pub async fn new(addr: SocketAddr, max: usize) -> NetResult<Self> {
        let max = max.max(1);
        let mut idle = Vec::with_capacity(max);
        for _ in 0..max {
            idle.push(Connection::connect(addr).await?);
        }
        Ok(ConnectionPool {
            addr,
            max,
            sem: Arc::new(Semaphore::new(max)),
            idle: Arc::new(Mutex::new(idle)),
        })
    }

    /// Acquire a connection from the pool with a 5-second timeout.
    pub async fn acquire(&self) -> NetResult<PooledConn<'_>> {
        let permit = timeout(ACQUIRE_TIMEOUT, self.sem.acquire())
            .await
            .map_err(|_| {
                NetError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "pool acquire timed out",
                ))
            })?
            .map_err(|_| {
                NetError::Io(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "pool semaphore closed",
                ))
            })?;

        let conn = {
            let mut guard = self.idle.lock().await;
            guard.pop()
        };

        let conn = match conn {
            Some(c) => c,
            None => Connection::connect(self.addr).await?,
        };

        Ok(PooledConn {
            conn: Some(conn),
            idle: Arc::clone(&self.idle),
            _permit: permit,
        })
    }

    /// Ping every idle connection, rebuild any that are broken.
    ///
    /// Returns the number of connections that responded successfully.
    pub async fn health_check(&self) -> usize {
        let mut guard = self.idle.lock().await;
        let mut healthy = 0usize;
        let mut rebuilt: Vec<Connection> = Vec::with_capacity(guard.len());
        for mut conn in guard.drain(..) {
            match conn.ping().await {
                Ok(_) => {
                    healthy += 1;
                    rebuilt.push(conn);
                }
                Err(_) => {
                    if let Ok(fresh) = Connection::connect(self.addr).await {
                        rebuilt.push(fresh);
                    }
                }
            }
        }
        *guard = rebuilt;
        healthy
    }

    /// Maximum number of connections in the pool.
    pub fn capacity(&self) -> usize {
        self.max
    }

    /// Number of idle (available) connections right now.
    pub async fn idle_count(&self) -> usize {
        self.idle.lock().await.len()
    }
}

/// RAII guard for a borrowed connection.
///
/// Dereferences to `&mut Connection` so callers can call `send_frame` /
/// `recv_frame` directly.  On drop the connection is returned to the pool.
pub struct PooledConn<'pool> {
    conn: Option<Connection>,
    idle: Arc<Mutex<Vec<Connection>>>,
    _permit: tokio::sync::SemaphorePermit<'pool>,
}

impl<'pool> std::ops::Deref for PooledConn<'pool> {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        self.conn.as_ref().expect("conn always Some while live")
    }
}

impl<'pool> std::ops::DerefMut for PooledConn<'pool> {
    fn deref_mut(&mut self) -> &mut Connection {
        self.conn.as_mut().expect("conn always Some while live")
    }
}

impl<'pool> Drop for PooledConn<'pool> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            let idle = Arc::clone(&self.idle);
            tokio::spawn(async move {
                idle.lock().await.push(conn);
            });
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use tokio::net::TcpListener;

    use super::*;
    use crate::frame::Opcode;

    async fn ping_ack_server(listener: TcpListener) {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let mut conn = Connection { stream, next_id: 1 };
                loop {
                    match conn.recv_frame().await {
                        Ok((_, _)) => {
                            if conn.send_frame(Opcode::PingAck, b"").await.is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    }

    async fn start_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(ping_ack_server(listener));
        (addr, handle)
    }

    #[tokio::test]
    async fn pool_new_succeeds() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 3).await.unwrap();
        assert_eq!(pool.capacity(), 3);
    }

    #[tokio::test]
    async fn pool_acquire_and_return() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 2).await.unwrap();
        let conn = pool.acquire().await.unwrap();
        assert_eq!(pool.capacity(), 2);
        drop(conn);
    }

    #[tokio::test]
    async fn pool_acquire_can_send_ping() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 1).await.unwrap();
        let mut conn = pool.acquire().await.unwrap();
        conn.send_frame(Opcode::Ping, b"").await.unwrap();
        let (hdr, _) = conn.recv_frame().await.unwrap();
        assert_eq!(hdr.opcode, Opcode::PingAck as u8);
    }

    #[tokio::test]
    async fn pool_capacity_is_respected() {
        let (addr, _srv) = start_server().await;
        let pool = Arc::new(ConnectionPool::new(addr, 2).await.unwrap());
        assert_eq!(pool.capacity(), 2);
    }

    #[tokio::test]
    async fn pool_health_check_returns_healthy_count() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 2).await.unwrap();
        let healthy = pool.health_check().await;
        assert_eq!(healthy, 2);
    }

    #[tokio::test]
    async fn pool_idle_count_decreases_on_acquire() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 3).await.unwrap();
        let idle_before = pool.idle_count().await;
        let _conn = pool.acquire().await.unwrap();
        let idle_after = pool.idle_count().await;
        assert!(idle_after < idle_before);
    }

    #[tokio::test]
    async fn pool_multiple_acquires_sequential() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 3).await.unwrap();
        for _ in 0..3 {
            let mut conn = pool.acquire().await.unwrap();
            conn.send_frame(Opcode::Ping, b"").await.unwrap();
            let _ = conn.recv_frame().await.unwrap();
        }
    }

    #[tokio::test]
    async fn pool_single_connection_pool() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 1).await.unwrap();
        assert_eq!(pool.capacity(), 1);
        let _conn = pool.acquire().await.unwrap();
    }

    #[tokio::test]
    async fn pool_capacity_respected_as_one() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 1).await.unwrap();
        assert_eq!(pool.capacity(), 1);
    }

    #[tokio::test]
    async fn pool_deref_mut_allows_send() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 1).await.unwrap();
        let mut conn = pool.acquire().await.unwrap();
        conn.send_frame(Opcode::Health, b"").await.unwrap();
        let (hdr, _) = conn.recv_frame().await.unwrap();
        assert_eq!(hdr.opcode, Opcode::PingAck as u8);
    }

    #[tokio::test]
    async fn pool_zero_capacity_treated_as_one() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 0).await.unwrap();
        assert_eq!(pool.capacity(), 1);
    }

    #[tokio::test]
    async fn pool_health_check_with_single_conn() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 1).await.unwrap();
        let healthy = pool.health_check().await;
        assert_eq!(healthy, 1);
    }

    #[tokio::test]
    async fn pool_acquire_returns_conn_on_drop() {
        let (addr, _srv) = start_server().await;
        let pool = ConnectionPool::new(addr, 1).await.unwrap();
        {
            let _conn = pool.acquire().await.unwrap();
        }
        // After drop, should be able to acquire again
        let _conn2 = pool.acquire().await.unwrap();
        assert_eq!(pool.capacity(), 1);
    }
}
