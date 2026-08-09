using System.Diagnostics;
using System.Net;
using System.Net.Sockets;
using VectorVolt.Client;
using VectorVolt.Client.Transport;

namespace VectorVolt.Client.Tests;

/// <summary>
/// Spawns a real <c>vvctl serve</c> process and tears it down on Dispose.
/// Tests use port 0 by asking the OS for a free port before launching.
/// </summary>
internal sealed class ServerHarness : IDisposable
{
    private readonly Process _proc;
    private readonly string _tempDir;
    private bool _disposed;

    public int Port { get; }
    public string KeyId  { get; }
    public string Secret { get; }

    private ServerHarness(Process proc, string tempDir, int port, string keyId, string secret)
    {
        _proc    = proc;
        _tempDir = tempDir;
        Port     = port;
        KeyId    = keyId;
        Secret   = secret;
    }

    /// <summary>
    /// Start a no-auth server with the given column schema.
    /// <paramref name="columns"/> must include the PK column as the first entry.
    /// </summary>
    public static ServerHarness Start(string[] columns, int timeoutMs = 5_000)
        => StartCore(columns, keyId: null, secret: null, timeoutMs);

    /// <summary>Start an authenticated server.</summary>
    public static ServerHarness StartWithAuth(
        string[] columns, string keyId, string secret, int timeoutMs = 5_000)
        => StartCore(columns, keyId, secret, timeoutMs);

    private static ServerHarness StartCore(
        string[] columns, string? keyId, string? secret, int timeoutMs)
    {
        int port    = GetFreePort();
        string dir  = Path.Combine(Path.GetTempPath(), $"vv_test_{Guid.NewGuid():N}");
        Directory.CreateDirectory(dir);

        string vvctl = VvctlPath();
        var args = new List<string>
        {
            "serve", dir,
            "--addr", $"127.0.0.1:{port}",
            "--columns"
        };
        args.AddRange(columns);
        if (secret != null)
        {
            args.Add("--secret"); args.Add(secret);
            args.Add("--key-id"); args.Add(keyId ?? "default");
        }

        var psi = new ProcessStartInfo
        {
            FileName               = vvctl,
            Arguments              = string.Join(" ", args),
            RedirectStandardOutput = true,
            RedirectStandardError  = true,
            UseShellExecute        = false,
            CreateNoWindow         = true,
        };

        var proc = Process.Start(psi)
            ?? throw new InvalidOperationException($"Failed to start vvctl: {vvctl}");

        // Wait until the port accepts a TCP connection — the server is ready.
        WaitForPort(port, timeoutMs, proc);

        return new ServerHarness(proc, dir, port, keyId ?? "", secret ?? "");
    }

    /// <summary>Open a client connected to this server. No auth.</summary>
    public VectorVoltClient OpenClient(int connectTimeoutMs = 3_000)
    {
        var transport = SocketTransport.Connect("127.0.0.1", Port, connectTimeoutMs);
        return new VectorVoltClient(transport);
    }

    /// <summary>Open an authenticated client.</summary>
    public VectorVoltClient OpenAuthClient(int connectTimeoutMs = 3_000)
    {
        if (string.IsNullOrEmpty(Secret))
            throw new InvalidOperationException("Harness was started without auth");
        return VectorVoltClient.Connect("127.0.0.1", Port, KeyId, Secret, connectTimeoutMs);
    }

    private static void WaitForPort(int port, int timeoutMs, Process proc)
    {
        var deadline = DateTime.UtcNow.AddMilliseconds(timeoutMs);
        while (DateTime.UtcNow < deadline)
        {
            if (proc.HasExited)
            {
                var stderr = proc.StandardError.ReadToEnd();
                throw new InvalidOperationException(
                    $"vvctl exited early (code {proc.ExitCode}): {stderr}");
            }
            try
            {
                using var s = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
                s.Connect(new IPEndPoint(IPAddress.Loopback, port));
                return; // connected — server is up
            }
            catch { Thread.Sleep(50); }
        }
        proc.Kill();
        throw new TimeoutException($"vvctl did not listen on port {port} within {timeoutMs} ms");
    }

    private static int GetFreePort()
    {
        using var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        int port = ((IPEndPoint)listener.LocalEndpoint).Port;
        listener.Stop();
        return port;
    }

    private static string VvctlPath()
    {
        // Resolve relative to the solution root: sdk/csharp/… → up 3 → repo root → vectorvolt/target/release/vvctl
        var dir = AppContext.BaseDirectory;
        for (int i = 0; i < 10; i++)
        {
            var candidate = Path.Combine(dir, "vectorvolt", "target", "release", "vvctl.exe");
            if (File.Exists(candidate)) return candidate;
            candidate = Path.Combine(dir, "vectorvolt", "target", "release", "vvctl");
            if (File.Exists(candidate)) return candidate;
            dir = Path.GetDirectoryName(dir) ?? dir;
        }
        throw new FileNotFoundException(
            "Could not find vectorvolt/target/release/vvctl[.exe]. " +
            "Run `cargo build --bin vvctl --release` in the vectorvolt/ directory first.");
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        try { if (!_proc.HasExited) _proc.Kill(); } catch { /* ignore */ }
        _proc.Dispose();
        try { Directory.Delete(_tempDir, recursive: true); } catch { /* ignore */ }
    }
}
