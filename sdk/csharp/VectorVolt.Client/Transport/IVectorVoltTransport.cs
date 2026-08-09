namespace VectorVolt.Client.Transport;

/// <summary>
/// Byte-level transport abstraction for the VectorVolt wire protocol.
///
/// <para>The client only sees two operations: write a contiguous buffer, read
/// exactly N bytes. Everything above this — framing, opcodes, CRC — is the
/// client's concern. Everything below — whether the bytes travel over TCP,
/// RDMA, or a shared-memory ring — is the transport's concern.</para>
///
/// <para>Implementations provided today:
/// <list type="bullet">
///   <item><see cref="SocketTransport"/> — raw <c>Socket</c>, io_uring on Linux
///   (.NET 7+ routes <c>Socket</c> I/O through io_uring automatically), TCP_NODELAY,
///   zero heap allocation on the send/receive hot path.</item>
/// </list>
/// </para>
///
/// <para>Planned transports (separate layers):
/// <list type="bullet">
///   <item><b>RdmaTransport</b> — P/Invoke into libibverbs; queue-pair per
///   connection; true kernel bypass once the Rust server exposes an RDMA
///   endpoint.</item>
///   <item><b>SharedMemoryTransport</b> — lock-free ring buffer in an mmap'd
///   region for same-host deployments; sub-microsecond round trips with no
///   network stack at all.</item>
/// </list>
/// </para>
/// </summary>
public interface IVectorVoltTransport : IDisposable
{
    /// <summary>
    /// Write all bytes in <paramref name="data"/> to the transport.
    /// Blocks until every byte is handed off. Loops internally on partial sends.
    /// </summary>
    void Send(ReadOnlySpan<byte> data);

    /// <summary>
    /// Read exactly <c>buffer.Length</c> bytes into <paramref name="buffer"/>.
    /// Blocks until the buffer is full. Loops internally on partial receives.
    /// Throws <see cref="VectorVoltException"/> if the connection is closed before
    /// the buffer is filled.
    /// </summary>
    void ReceiveExact(Span<byte> buffer);
}
