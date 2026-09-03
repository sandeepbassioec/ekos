using VectorVolt.Client;
using VectorVolt.Client.Transport;
using Xunit;
using System;
using System.Collections.Generic;

namespace VectorVolt.Client.Tests;

/// <summary>
/// Unit tests for C# SDK completeness (VV-SDK-002).
/// Uses mocks — no live server required.
/// </summary>
public class SdkCompletionTests
{
    private class ThrowingTransport : IVectorVoltTransport
    {
        public void Send(ReadOnlySpan<byte> data) => throw new System.Net.Sockets.SocketException();
        public void ReceiveExact(Span<byte> buffer) => throw new System.Net.Sockets.SocketException();
        public void Dispose() { }
    }

    [Fact]
    public void Client_Constructor_DoesNotThrow()
    {
        var t = new ThrowingTransport();
        var client = new VectorVoltClient(t);
        Assert.NotNull(client);
    }

    [Fact]
    public void Client_Dispose_DoesNotThrow()
    {
        var t = new ThrowingTransport();
        var client = new VectorVoltClient(t);
        client.Dispose();
    }

    [Fact]
    public void Client_DoubleDispose_DoesNotThrow()
    {
        var t = new ThrowingTransport();
        var client = new VectorVoltClient(t);
        client.Dispose();
        client.Dispose();
    }

    [Fact]
    public void Client_NullTransport_ThrowsArgumentNullException()
    {
        Assert.Throws<ArgumentNullException>(() => new VectorVoltClient(null!));
    }

    [Fact]
    public void Ping_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.Ping());
    }

    [Fact]
    public void Stats_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.Stats());
    }

    [Fact]
    public void Insert_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.Insert(1UL, new ulong[] { 100UL }));
    }

    [Fact]
    public void Get_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.Get(1UL));
    }

    [Fact]
    public void Delete_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.Delete(1UL));
    }

    [Fact]
    public void BeginTxn_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.BeginTxn());
    }

    [Fact]
    public void CommitTxn_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.CommitTxn(1UL));
    }

    [Fact]
    public void RollbackTxn_ThrowsOnUnreachable()
    {
        var client = new VectorVoltClient(new ThrowingTransport());
        Assert.ThrowsAny<Exception>(() => client.RollbackTxn(1UL));
    }
}
