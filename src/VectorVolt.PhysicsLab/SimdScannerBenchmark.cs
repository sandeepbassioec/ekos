using BenchmarkDotNet.Attributes;
using System;
using System.Collections.Generic;
using System.Numerics;
using System.Runtime.Intrinsics;
using System.Runtime.Intrinsics.X86;
using System.Text;

namespace VectorVolt.PhysicsLab
{
    public class SimdScannerBenchmark
    {
        private const int TotalAccounts = 10_000_000;
        private int[] _memoryArena;
        private int _targetAccountId;

        [GlobalSetup]
        public void Setup()
        {
            _memoryArena = new int[TotalAccounts];

            // Populate the arena
            for (int i = 0; i < TotalAccounts; i++)
            {
                _memoryArena[i] = i;
            }

            // We put the target near the very end so the scanner has to work for it.
            _targetAccountId = 9_999;
        }

        // ---------------------------------------------------------
        // 1. THE STANDARD WAY (Scalar Execution)
        // 1 Integer compared per clock cycle.
        // ---------------------------------------------------------
        [Benchmark(Baseline = true)]
        public int StandardForLoop()
        {
            for (int i = 0; i < _memoryArena.Length; i++)
            {
                if (_memoryArena[i] == _targetAccountId)
                {
                    return i;
                }
            }
            return -1;
        }

        // ---------------------------------------------------------
        // 2. THE VECTORVOLT WAY (SIMD AVX2 + Bit Manipulation)
        // 8 Integers compared per clock cycle!
        // ---------------------------------------------------------
        [Benchmark]
        public unsafe int Avx2VectorScan()
        {
            // Safety Check: Does this physical CPU have AVX2 circuits?
            if (!Avx2.IsSupported) return StandardForLoop();

            // STEP 1: Create a 256-bit hardware register containing 8 copies of our Target ID
            Vector256<int> targetVector = Vector256.Create(_targetAccountId);

            // We use 'fixed' to pin the array in RAM so the Garbage Collector 
            // doesn't move it while we are doing pointer math.
            fixed (int* pointer = _memoryArena)
            {
                // We jump by 8 in our loop because we process 8 items at a time!
                for (int i = 0; i < TotalAccounts; i += 8)
                {
                    // STEP 2: Load 8 accounts from RAM into the CPU in a single instruction.
                    Vector256<int> memoryChunk = Avx.LoadVector256(pointer + i);

                    // STEP 3: Hardware Comparison. 
                    // This compares all 8 slots simultaneously. 
                    Vector256<int> result = Avx2.CompareEqual(targetVector, memoryChunk);

                    // STEP 4: Convert the 256-bit vector result into a standard integer Bitmask.
                    // If no matches, the mask is 0. 
                    // If the 3rd item matched, the mask is 0000_1000.
                    int mask = Avx2.MoveMask(result.AsByte());

                    if (mask != 0)
                    {
                        // STEP 5: BMI Circuit! Count the trailing zeroes in the mask 
                        // to find the exact array index without looping!
                        // (We divide by 4 because each integer is 4 bytes).
                        int exactIndex = i + (BitOperations.TrailingZeroCount((uint)mask) / 4);
                        return exactIndex;
                    }
                }
            }
            return -1;
        }
    }
}
