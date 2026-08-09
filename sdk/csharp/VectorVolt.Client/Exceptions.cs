namespace VectorVolt.Client;

/// <summary>Base exception for all VectorVolt client errors.</summary>
public class VectorVoltException : Exception
{
    public VectorVoltException(string message) : base(message) { }
    public VectorVoltException(string message, Exception inner) : base(message, inner) { }
}

/// <summary>Server returned an error response (opcode 0xFF with a message payload).</summary>
public sealed class VectorVoltServerException : VectorVoltException
{
    public byte ErrorCode { get; }

    public VectorVoltServerException(byte errorCode, string message)
        : base(message)
    {
        ErrorCode = errorCode;
    }
}

/// <summary>Authentication failed — wrong secret or nonce replay.</summary>
public sealed class VectorVoltAuthException : VectorVoltException
{
    public VectorVoltAuthException(string message) : base(message) { }
}

/// <summary>Protocol violation — unexpected opcode, bad magic, or CRC mismatch.</summary>
public sealed class VectorVoltProtocolException : VectorVoltException
{
    public VectorVoltProtocolException(string message) : base(message) { }
}
