using BenchmarkDotNet.Attributes;
using System;
using System.Collections.Generic;
using System.Numerics;
using System.Runtime.Intrinsics;
using System.Runtime.Intrinsics.X86;
using System.Text;

namespace VectorVolt.PhysicsLab
{
    public class CommaHunterBenchmark
    {
        // 32 Megabytes of data (A massive network payload)
        private const int PayloadSize = 32 * 1024 * 1024;
        private byte[] _networkStream;

        [GlobalSetup]
        public void Setup()
        {
            _networkStream = new byte[PayloadSize];

            // Fill the stream with random text data (using 'A' as dummy data)
            Array.Fill(_networkStream, (byte)'A');

            // Hide the comma near the very end of the 32MB payload
            // This forces the CPU to scan almost the entire array.
            _networkStream[PayloadSize - 64] = (byte)',';
        }

        // ---------------------------------------------------------
        // 1. THE OLD FASHION WAY
        // Branching logic. The CPU evaluates an 'if' statement 33 million times.
        // ---------------------------------------------------------
        [Benchmark(Baseline = true)]
        public int FindCommaStandardLoop()
        {
            for (int i = 0; i < _networkStream.Length; i++)
            {
                if (_networkStream[i] == (byte)',') // The Branch Predictor Trap
                {
                    return i;
                }
            }
            return -1;
        }

        // ---------------------------------------------------------
        // 2. THE INTRINSICS WAY (AVX2 + BMI1)
        // Zero branching. 32 bytes compared simultaneously per cycle.
        // ---------------------------------------------------------
        [Benchmark]
        public unsafe int FindCommaIntrinsics()
        {
            if (!Avx2.IsSupported) return FindCommaStandardLoop();

            // Create a 256-bit register filled with 32 commas
            Vector256<byte> targetComma = Vector256.Create((byte)',');

            fixed (byte* ptr = _networkStream)
            {
                // We jump by 32 bytes every loop!
                for (int i = 0; i < _networkStream.Length - 32; i += 32)
                {
                    // 1. Load 32 bytes from RAM instantly
                    Vector256<byte> chunk = Avx.LoadVector256(ptr + i);

                    // 2. Hardware comparison of all 32 bytes
                    Vector256<byte> match = Avx2.CompareEqual(targetComma, chunk);

                    // 3. Compress the 32-byte result into a 32-bit integer mask
                    int mask = Avx2.MoveMask(match);

                    // 4. If the mask is anything other than 0, we found it!
                    if (mask != 0)
                    {
                        // 5. BMI1 Hardware Circuit: Instantly find the exact index of the '1' bit
                        int exactIndex = i + BitOperations.TrailingZeroCount((uint)mask);
                        return exactIndex;
                    }
                }
            }
            return -1;
        }
    }
}
