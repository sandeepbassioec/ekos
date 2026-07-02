using System;
using System.Collections.Generic;
using System.Text;

namespace VectorVolt.PhysicsLab;

// 1. NAMESPACE DECLARATIONS - What we're importing and WHY
using BenchmarkDotNet.Attributes;     // [Benchmark], [GlobalSetup] attributes - marks methods to measure
using BenchmarkDotNet.Running;        // BenchmarkRunner.Run() - starts the measurement process
using System.Runtime.CompilerServices; // [MethodImpl] - controls JIT behavior (optional optimization hints)
using System.Runtime.Intrinsics;       // Vector128<T> - the 128-bit container that holds multiple values
using System.Runtime.Intrinsics.X86;   // Sse class - the actual hardware instruction wrappers

[ShortRunJob]                         // Runs only 3 iterations instead of 15+ (good for learning/quick feedback)
[DisassemblyDiagnoser(printSource: true, maxDepth: 3)]  // SHOWS YOU THE ACTUAL CPU INSTRUCTIONS!
[MemoryDiagnoser]                     // Tracks GC allocations (SIMD typically produces ZERO allocations)
public class SseAddBenchmark
{
    // 2. CONSTANTS AND FIELDS
    // Why 4096? 
    // - Large enough to measure meaningful performance difference
    // - Small enough to fit in L2/L3 CPU cache (avoids RAM latency skewing results)
    // - Multiple of 4 (Vector128<float>.Count) so no remainder handling needed for clean comparison
    private const int ElementCount = 4097;

    // Arrays are stored on the HEAP (managed memory)
    // In the "unsafe" version, we'll pin them to get raw pointers
    private readonly float[] _dataA = new float[ElementCount];
    private readonly float[] _dataB = new float[ElementCount];
    private readonly float[] _result = new float[ElementCount];

    // 3. SETUP METHOD - Runs ONCE before any benchmarks start
    [GlobalSetup]
    public void Setup()
    {
        // Initialize with deterministic but non-trivial data
        // This prevents the JIT from doing "constant folding" optimization
        for (int i = 0; i < ElementCount; i++)
        {
            _dataA[i] = i * 0.1f;                    // 0.0, 0.1, 0.2, 0.3 ...
            _dataB[i] = (ElementCount - i) * 0.01f;  // 40.96, 40.95, 40.94 ...
        }
        // Result array doesn't need initialization - we'll overwrite everything
    }

    // ================================================================
    // BENCHMARK 1: THE "OLD FASHIONED" WAY (Our Baseline)
    // ================================================================
    [Benchmark(Baseline = true)]  // Baseline = true means this becomes 1.00x reference point
    public void NaiveScalarAddition()
    {
        // LINE 1: Standard C# for loop
        // What happens here:
        // - The JIT compiler checks array bounds on EVERY iteration (bounds check elimination may apply)
        // - Each iteration processes exactly ONE float (32 bits)
        // - CPU executes scalar instructions: vaddss (add scalar single)
        for (int i = 0; i < ElementCount; i++)
        {
            // LINE 2: Scalar addition
            // This compiles to: vaddss xmm0, xmm1, xmm2  (on x86)
            //                  fadd s0, s1, s2           (on ARM)
            // Key concept: Uses only the LOWEST 32 bits of the 128-bit XMM register
            _result[i] = _dataA[i] + _dataB[i];
        }
    }

    // ================================================================
    // BENCHMARK 2: SSE SIMD VERSION (Managed Array Access)
    // ================================================================
    [Benchmark]
    public void SseVectorAddition()
    {
        // LINE 1: Feature check - ESSENTIAL for production code
        // Sse.IsSupported is a JIT-time constant on .NET Core 3.0+
        // If false, the JIT will completely DELETE this if-block body
        if (!Sse.IsSupported)
            throw new NotSupportedException("SSE not supported on this machine");

        int i = 0;

        // LINE 2: Loop condition explained
        // Vector128<float>.Count = 4 (because 128 bits / 32 bits per float = 4)
        // We subtract Count so we don't read past the end of the array
        // Example: ElementCount = 4096, Count = 4
        // Loop runs while i <= 4092 (i.e., 4092, 4088, 4084 ... 0)
        for (; i <= ElementCount - Vector128<float>.Count; i += Vector128<float>.Count)
        {
            // LINE 3-4: Creating vectors from array elements
            // THIS IS THE SLOW PART of this method (array indexing overhead)
            // Vector128.Create() has 4 overloads: 2,3,4, or Span parameters
            // Each array access here has bounds checking!
            var va = Vector128.Create(
                _dataA[i],     // array index access
                _dataA[i + 1], // array index access  
                _dataA[i + 2], // array index access
                _dataA[i + 3]  // array index access
            );
            var vb = Vector128.Create(
                _dataB[i],
                _dataB[i + 1],
                _dataB[i + 2],
                _dataB[i + 3]
            );

            // LINE 5: THE INTRINSIC CALL - This is what we're here to learn!
            // Sse.Add(Vector128<float>, Vector128<float>) -> Vector128<float>
            // 
            // WHAT HAPPENS AT CPU LEVEL:
            // Input: Two 128-bit XMM registers (each holding 4 floats)
            // Output: One 128-bit XMM register (4 summed floats)
            // CPU Instruction: ADDPS xmm0, xmm1  (Add Packed Single-Precision)
            // 
            // Visual representation of what happens inside the CPU:
            // XMM0: [a3, a2, a1, a0]
            // XMM1: [b3, b2, b1, b0]
            // XMM0 = XMM0 + XMM1
            // Result: [a3+b3, a2+b2, a1+b1, a0+b0]
            // ALL 4 ADDITIONS HAPPEN IN A SINGLE CLOCK CYCLE!
            var vresult = Sse.Add(va, vb);

            // LINE 6-9: Extracting results back to array (ALSO SLOW)
            // GetElement() extracts a single float from the vector
            // Each call costs a scalar store instruction
            _result[i] = vresult.GetElement(0);
            _result[i + 1] = vresult.GetElement(1);
            _result[i + 2] = vresult.GetElement(2);
            _result[i + 3] = vresult.GetElement(3);
        }

        // LINE 10-13: Handle remainder elements (scalar fallback)
        // For our array size (4096), this loop NEVER executes
        // But it's required for arbitrary array sizes
        for (; i < ElementCount; i++)
        {
            _result[i] = _dataA[i] + _dataB[i];
        }
    }

    // ================================================================
    // BENCHMARK 3: UNSAFE POINTER VERSION (True SIMD Performance)
    // ================================================================
    [Benchmark]
    public unsafe void SseVectorAdditionUnsafe()
    {
        if (!Sse.IsSupported)
            throw new NotSupportedException("SSE not supported");

        // LINE 1: The 'fixed' statement - PINNING memory
        // 
        // WHAT: Prevents GC from moving the array during compaction
        // WHY: SIMD load/store instructions need STABLE memory addresses
        // HOW: Creates a pinned GC handle temporarily
        // 
        // Without 'fixed': GC could move array -> pointer becomes invalid -> CRASH
        // With 'fixed': GC cannot move array -> pointer stays valid -> SAFE
        fixed (float* pA = _dataA, pB = _dataB, pRes = _result)
        {
            // LINE 2: Pointer arithmetic preparation
            // pA points to _dataA[0] (start of array)
            float* aPtr = pA;
            float* bPtr = pB;
            float* resPtr = pRes;

            // LINE 3: Loop through array in chunks of 4
            for (int i = 0; i < ElementCount; i += Vector128<float>.Count)
            {
                // LINE 4-5: SIMD LOAD INSTRUCTIONS
                // Sse.LoadVector128(float* address) -> Vector128<float>
                // 
                // CPU Instruction: MOVAPS xmm0, [address] (Move Aligned Packed Single)
                // 
                // aPtr + i calculates memory offset:
                // i=0: load from &_dataA[0]
                // i=4: load from &_dataA[4]
                // 
                // KEY BENEFIT: NO BOUNDS CHECKING, NO INDEX CALCULATION
                // Just raw memory read - exactly what the CPU wants
                var va = Sse.LoadVector128(aPtr + i);
                var vb = Sse.LoadVector128(bPtr + i);

                // LINE 6: The exact same intrinsic as before
                var vresult = Sse.Add(va, vb);

                // LINE 7: SIMD STORE INSTRUCTION
                // Sse.Store(float* address, Vector128<float> value)
                // 
                // CPU Instruction: MOVAPS [address], xmm0 (Move Aligned Packed Single)
                // 
                // Writes all 4 floats back to memory in ONE instruction
                Sse.Store(resPtr + i, vresult);
            }
        }
        // LINE 8: After closing brace, the 'fixed' statement UNPINS the arrays
        // GC is now free to move them if needed
    }

    // ================================================================
    // BENCHMARK 4: VERIFICATION METHOD (Not a benchmark, just validation)
    // ================================================================
    // [Benchmark] - Commented out because this is for correctness checking
    public bool VerifyResults()
    {
        // Run the naive version as our "ground truth"
        NaiveScalarAddition();
        float[] expected = (float[])_result.Clone();

        // Run the SIMD version
        SseVectorAdditionUnsafe();

        // Compare every element (using tolerance because floating point math)
        for (int i = 0; i < ElementCount; i++)
        {
            if (Math.Abs(expected[i] - _result[i]) > 0.0001f)
                return false;
        }
        return true;
    }
}

//// ================================================================
//// ENTRY POINT - What runs when you press F5
//// ================================================================
//public class Program
//{
//    public static void Main(string[] args)
//    {
//        // This single line runs ALL benchmarks in the assembly
//        // It will:
//        // 1. Compile each method
//        // 2. Warm up the JIT
//        // 3. Run multiple iterations
//        // 4. Calculate statistics
//        // 5. Generate disassembly
//        // 6. Output a beautiful table
//        var summary = BenchmarkRunner.Run<SseAddBenchmark>();

//        // Optional: Verify correctness first
//        var bench = new SseAddBenchmark();
//        bench.Setup();
//        Console.WriteLine($"Verification: {bench.VerifyResults()}");
//    }
//}