using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
using System;

namespace VectorVolt.PhysicsLab
{
    // CRITICAL: This attribute tells BenchmarkDotNet to track the Garbage Collector!
    [MemoryDiagnoser]
    public class GcAllocationBenchmark
    {
        private const int Operations = 100_000;

        // ---------------------------------------------------------
        // 1. THE STANDARD WAY (Heap Allocation)
        // Developers love classes and strings. Every time we use 'new', 
        // we ask the OS for fresh memory on the Managed Heap.
        // ---------------------------------------------------------
        public class UserCommandClass
        {
            public int UserId { get; set; }
            public string Action { get; set; }
        }

        [Benchmark(Baseline = true)]
        public int ProcessWithClasses()
        {
            int dummyResult = 0;
            for (int i = 0; i < Operations; i++)
            {
                // THE FATAL FLAW: Creating a new object inside a hot loop.
                var cmd = new UserCommandClass();
                cmd.UserId = i;
                cmd.Action = "Update"; // String reference

                dummyResult += cmd.UserId;
            }
            return dummyResult; // The 100,000 objects are now "Garbage" left on the Heap.
        }

        // ---------------------------------------------------------
        // 2. THE VECTORVOLT WAY (Zero Allocation / Pre-allocated Arena)
        // We use structs, no strings, and we allocate the memory exactly ONCE 
        // when the OS boots up.
        // ---------------------------------------------------------
        public struct UserCommandStruct
        {
            public int UserId;
            public int ActionCode; // 1 = Update. No strings allowed!
        }

        // The Memory Arena: Pre-allocated BEFORE the benchmark runs.
        private UserCommandStruct[] _arena = new UserCommandStruct[Operations];

        [Benchmark]
        public int ProcessWithStructs()
        {
            int dummyResult = 0;
            for (int i = 0; i < Operations; i++)
            {
                // ZERO ALLOCATION: We do not use 'new'. 
                // We just overwrite the existing electrons in the silicon.
                _arena[i].UserId = i;
                _arena[i].ActionCode = 1;

                dummyResult += _arena[i].UserId;
            }
            return dummyResult; // No garbage was created. The GC stays asleep.
        }
    }
}