using BenchmarkDotNet.Attributes;
using System;
using System.Collections.Generic;
using System.Runtime.Intrinsics;
using System.Runtime.Intrinsics.X86;
using System.Text;

namespace VectorVolt.PhysicsLab;

[ShortRunJob] // Good for quick feedback during learning
[DisassemblyDiagnoser(printSource: true, maxDepth: 3)] // Shows generated ASM
[MemoryDiagnoser]
public class TemplateBenchmark
{
    // === Configuration ===
    // Adjust size to be large enough to matter, small enough to stay in L2/L3 cache
    private const int ElementCount = 4096;
    private readonly float[] _dataA = new float[ElementCount];
    private readonly float[] _dataB = new float[ElementCount];
    private readonly float[] _result = new float[ElementCount];

    [GlobalSetup]
    public void Setup()
    {
        // Fill with deterministic but random-looking data
        for (int i = 0; i < ElementCount; i++)
        {
            _dataA[i] = i * 0.1f;
            _dataB[i] = (ElementCount - i) * 0.01f;
        }
    }

    // --- 1. NAIVE BASELINE (Old Fashioned C#) ---
    [Benchmark(Baseline = true)]
    public void NaiveOperation()
    {
        for (int i = 0; i < ElementCount; i++)
        {
            // Replace with the specific operation you are testing
            _result[i] = _dataA[i] + _dataB[i];
        }
    }

    // --- 2. INTRINSICS IMPLEMENTATION ---
    [Benchmark]
    public void IntrinsicsOperation()
    {
        // Safety check: If your CPU doesn't support this, BenchmarkDotNet will warn you.
        if (!Sse.IsSupported)
            throw new NotSupportedException("SSE not supported on this machine");

        int i = 0;
        // Vector128<float> holds 4 floats at a time
        for (; i <= ElementCount - Vector128<float>.Count; i += Vector128<float>.Count)
        {
            // Load data
            var va = Vector128.Create(_dataA[i], _dataA[i + 1], _dataA[i + 2], _dataA[i + 3]);
            var vb = Vector128.Create(_dataB[i], _dataB[i + 1], _dataB[i + 2], _dataB[i + 3]);

            // *** THE SPECIFIC INTRINSIC METHOD BEING TESTED ***
            var vresult = Sse.Add(va, vb);
            // ********************************************

            // Store result
            _result[i] = vresult.GetElement(0);
            _result[i + 1] = vresult.GetElement(1);
            _result[i + 2] = vresult.GetElement(2);
            _result[i + 3] = vresult.GetElement(3);
        }

        // Handle remaining elements (scalar loop)
        for (; i < ElementCount; i++)
        {
            _result[i] = _dataA[i] + _dataB[i];
        }
    }

    // --- 3. ADVANCED: Unsafe Pointer Version (Closer to Metal) ---
    // This removes array bounds checks and shows true instruction latency.
    [Benchmark]
    public unsafe void IntrinsicsUnsafeOperation()
    {
        if (!Sse.IsSupported)
            throw new NotSupportedException("SSE not supported");

        fixed (float* pA = _dataA, pB = _dataB, pRes = _result)
        {
            float* aPtr = pA;
            float* bPtr = pB;
            float* resPtr = pRes;

            for (int i = 0; i < ElementCount; i += Vector128<float>.Count)
            {
                var va = Sse.LoadVector128(aPtr + i);
                var vb = Sse.LoadVector128(bPtr + i);

                var vresult = Sse.Add(va, vb);

                Sse.Store(resPtr + i, vresult);
            }
        }
    }
}