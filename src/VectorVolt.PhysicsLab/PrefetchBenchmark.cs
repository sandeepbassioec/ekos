using BenchmarkDotNet.Attributes;
using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Text;

namespace VectorVolt.PhysicsLab
{
    public class PrefetchBenchmark
    {
        // 10 Million accounts = ~40 Megabytes of data. 
        // This is much larger than the L3 cache, guaranteeing we suffer slow RAM fetches.
        private const int AccountCount = 10_000_000;
        private const int CommandCount = 1_000_000;

        private int[] _database;
        private int[] _randomCommands;

        [GlobalSetup]
        public void Setup()
        {
            _database = new int[AccountCount];
            _randomCommands = new int[CommandCount];
            var rand = new Random(42); // Seeded for consistent benchmarks

            // Populate the database
            for (int i = 0; i < AccountCount; i++)
            {
                _database[i] = i;
            }

            // Generate 1 million commands hitting completely random accounts
            for (int i = 0; i < CommandCount; i++)
            {
                _randomCommands[i] = rand.Next(0, AccountCount);
            }
        }

        // ---------------------------------------------------------
        // 1. THE STANDARD WAY (Blind Execution)
        // The CPU stalls on almost every single loop iteration waiting for RAM.
        // ---------------------------------------------------------
        [Benchmark(Baseline = true)]
        public int ProcessWithoutPrefetch()
        {
            int dummySum = 0;

            for (int i = 0; i < CommandCount; i++)
            {
                int targetAccountId = _randomCommands[i];

                // CACHE MISS: The CPU freezes here for ~100 nanoseconds!
                dummySum += _database[targetAccountId];
            }

            return dummySum;
        }

        // ---------------------------------------------------------
        // 2. THE VECTORVOLT WAY (Lookahead Pipelining)
        // We instruct the CPU to fetch data 16 steps in the future while 
        // it processes the current step.
        // ---------------------------------------------------------
        [Benchmark]
        public unsafe int ProcessWithSoftwarePrefetch()
        {
            int dummySum = 0;
            const int Lookahead = 16;

            for (int i = 0; i < CommandCount - Lookahead; i++)
            {
                // STEP 1: Look into the future.
                int futureAccountId = _randomCommands[i + Lookahead];
                ref int futureData = ref _database[futureAccountId];

                // STEP 2: Issue the raw hardware prefetch instruction! 
                // We check if the silicon supports SSE (Intel/AMD), then cast the ref to a raw pointer.
                if (System.Runtime.Intrinsics.X86.Sse.IsSupported)
                {
                    System.Runtime.Intrinsics.X86.Sse.Prefetch0(Unsafe.AsPointer(ref futureData));
                }

                //ref int futureData = ref _database[futureAccountId];
                //System.Runtime.Intrinsics.X86.Sse42.Unsafe.Prefetch0(ref futureData);
                // STEP 3: Process the CURRENT data. 
                int currentAccountId = _randomCommands[i];
                dummySum += _database[currentAccountId];
            }

            // Cleanup: Process the final 16 items normally since we can't look ahead anymore.
            for (int i = CommandCount - Lookahead; i < CommandCount; i++)
            {
                dummySum += _database[_randomCommands[i]];
            }

            return dummySum;
        }
    }
}
