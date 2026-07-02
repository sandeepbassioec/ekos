using BenchmarkDotNet.Attributes;
using System;
using System.Buffers.Binary;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Runtime.Intrinsics;
using System.Runtime.Intrinsics.X86;
using System.Text;

namespace VectorVolt.PhysicsLab
{
    public class MemoryShufflerBenchmark
    {
        // 32 Megabytes of Network Data (8 Million Integers)
        private const int PayloadSize = 32 * 1024 * 1024;
        private byte[] _networkStream;
        private byte[] _outputStream1;
        private byte[] _outputStream2;

        [GlobalSetup]
        public void Setup()
        {
            _networkStream = new byte[PayloadSize];
            _outputStream1 = new byte[PayloadSize];
            _outputStream2 = new byte[PayloadSize];

            // Fill the stream with random dummy data
            new Random(42).NextBytes(_networkStream);
        }

        // ---------------------------------------------------------
        // 1. THE OLD FASHION WAY
        // 1 Integer at a time. The CPU does bitwise math (shifts/ORs) 
        // 8 million times.
        // ---------------------------------------------------------
        [Benchmark(Baseline = true)]
        public void StandardEndianSwap()
        {
            // We treat the byte array as an array of 32-bit unsigned integers
            // Create Span<byte> locals from the arrays first to avoid taking
            // references directly to the readonly field which can trigger
            // CS8331 when the runtime produces refs into the span.
            var inputBytes = _networkStream.AsSpan();
            var outputBytes = _outputStream1.AsSpan();
            var inputSpan = MemoryMarshal.Cast<byte, uint>(inputBytes);
            var outputSpan = MemoryMarshal.Cast<byte, uint>(outputBytes);

            for (int i = 0; i < inputSpan.Length; i++)
            {
                // This C# method translates to the standard BSWAP scalar instruction
                outputSpan[i] = BinaryPrimitives.ReverseEndianness(inputSpan[i]);
            }
        }

        // ---------------------------------------------------------
        // 2. THE INTRINSICS WAY (AVX2 Shuffle)
        // 8 Integers at a time. The CPU physically crosses the wires 
        // inside the register in 1 clock cycle.
        // ---------------------------------------------------------
        [Benchmark]
        public unsafe void Avx2ShuffleSwap()
        {
            if (!Avx2.IsSupported) return;

            // THE CONTROL MASK:
            // VPSHUFB operates on two 128-bit (16-byte) lanes independently.
            // We tell the silicon exactly where each byte should go.
            // Look closely at the pattern: 0 becomes 3, 1 becomes 2, etc.
            var maskArray = new byte[]
            {
                3, 2, 1, 0,  7, 6, 5, 4,  11, 10, 9, 8,  15, 14, 13, 12, // Lower 16 bytes
                3, 2, 1, 0,  7, 6, 5, 4,  11, 10, 9, 8,  15, 14, 13, 12  // Upper 16 bytes
            };

            Vector256<byte> shuffleMask;
            fixed (byte* maskPtr = maskArray)
            {
                shuffleMask = Avx.LoadVector256(maskPtr);
            }

            fixed (byte* inPtr = _networkStream)
            fixed (byte* outPtr = _outputStream2)
            {
                // We jump by 32 bytes every loop!
                for (int i = 0; i < PayloadSize; i += 32)
                {
                    // STEP 1: Load 32 bytes (8 integers) from RAM
                    Vector256<byte> dataChunk = Avx.LoadVector256(inPtr + i);

                    // STEP 2: THE SHUFFLE!
                    // Apply the control mask. The bytes are instantly swapped.
                    Vector256<byte> reversedChunk = Avx2.Shuffle(dataChunk, shuffleMask);

                    // STEP 3: Blast the corrected bytes back to the output RAM
                    Avx.Store(outPtr + i, reversedChunk);
                }
            }
        }
    }
}
