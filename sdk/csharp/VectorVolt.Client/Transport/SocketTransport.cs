using System.Net;
using System.Net.Sockets;

namespace VectorVolt.Client.Transport;

/// <summary>
/// TCP transport backed by a raw <see cref="Socket"/>.
///
/// <para><b>Why <c>Socket</c> and not <c>TcpClient</c>?</b>
/// <c>TcpClient</c> wraps <c>Socket</c> with extra allocations, a
/// <c>NetworkStream</c> wrapper that adds another indirection, and no access
/// to the socket options needed for low-latency I/O. Using <c>Socket</c>
/// directly gives us:</para>
/// <list type="bullet">
///   <item><c>TCP_NODELAY</c> — disable Nagle's algorithm. A request/response
///   protocol has nothing to coalesce; Nagle adds up to 40 ms on small
///   frames.</item>
///   <item>Explicit send/receive buffer sizes tuned to the VectorVolt frame
///   sizes rather than the OS default.</item>
///   <item>Span-based <c>Send</c>/<c>Receive</c> overloads — no intermediate
///   byte array copy.</item>
///   <item>On Linux with .NET 7+, the runtime routes <c>Socket</c> I/O
///   through io_uring automatically: completions are batched in the kernel
///   and delivered without a syscall per I/O.</item>
/// </list>
/// </summary>
public sealed class SocketTransport : IVectorVoltTransport
{
    private readonly Socket _socket;
    private bool _disposed;

    // Tuned to typical VectorVolt frame sizes:
    //   header = 24 bytes, most payloads < 64 KB, batch payloads up to 16 MB.
    // The OS doubles whatever we set, so these are floor values.
    private const int SendBufferBytes    = 256 * 1024;
    private const int ReceiveBufferBytes = 256 * 1024;

    private SocketTransport(Socket socket)
    {
        _socket = socket;
    }

    /// <summary>Connect to a VectorVolt server.</summary>
    /// <param name="host">Hostname or IP address.</param>
    /// <param name="port">Port number.</param>
    /// <param name="connectTimeoutMs">
    /// How long to wait for the TCP handshake before throwing.
    /// Defaults to 5 seconds. Pass 0 for no timeout (uses OS default, which
    /// can be over a minute on an unreachable host).
    /// </param>
    public static SocketTransport Connect(string host, int port, int connectTimeoutMs = 5_000)
    {
        var endpoints = Dns.GetHostAddresses(host);
        if (endpoints.Length == 0)
            throw new VectorVoltException($"Could not resolve host '{host}'");

        // Prefer IPv4 to match the Rust server default binding.
        var addr = Array.Find(endpoints, e => e.AddressFamily == AddressFamily.InterNetwork)
                   ?? endpoints[0];

        var socket = new Socket(addr.AddressFamily, SocketType.Stream, ProtocolType.Tcp);

        // Disable Nagle — request/response has nothing to coalesce.
        socket.NoDelay = true;
        socket.SendBufferSize    = SendBufferBytes;
        socket.ReceiveBufferSize = ReceiveBufferBytes;

        if (connectTimeoutMs > 0)
        {
            // ConnectAsync with a timeout is the only clean way to time-box
            // the TCP handshake on all platforms.
            using var cts = new CancellationTokenSource(connectTimeoutMs);
            try
            {
                socket.ConnectAsync(new IPEndPoint(addr, port), cts.Token)
                      .AsTask().GetAwaiter().GetResult();
            }
            catch (OperationCanceledException)
            {
                socket.Dispose();
                throw new VectorVoltException(
                    $"Connection to {host}:{port} timed out after {connectTimeoutMs} ms");
            }
            catch
            {
                socket.Dispose();
                throw;
            }
        }
        else
        {
            socket.Connect(addr, port);
        }

        return new SocketTransport(socket);
    }

    /// <inheritdoc/>
    public void Send(ReadOnlySpan<byte> data)
    {
        while (data.Length > 0)
        {
            int sent = _socket.Send(data, SocketFlags.None);
            if (sent == 0)
                throw new VectorVoltException("Connection closed by server during send");
            data = data.Slice(sent);
        }
    }

    /// <inheritdoc/>
    public void ReceiveExact(Span<byte> buffer)
    {
        while (buffer.Length > 0)
        {
            int received = _socket.Receive(buffer, SocketFlags.None);
            if (received == 0)
                throw new VectorVoltException("Connection closed by server during receive");
            buffer = buffer.Slice(received);
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        try { _socket.Shutdown(SocketShutdown.Both); } catch { /* ignore */ }
        _socket.Dispose();
    }
}
