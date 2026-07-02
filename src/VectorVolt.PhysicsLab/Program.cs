using BenchmarkDotNet.Attributes;
using BenchmarkDotNet.Running;
using System.Runtime.InteropServices;
using VectorVolt.PhysicsLab;

//Console.WriteLine("Hello, World!");
// ---------------------------------------------------------
// 1. THE STANDARD WAY (How 99% of developers write it)
// The compiler guesses the size. It usually ends up being 65 or 68 bytes.
// This forces the CPU to read TWO 64-byte Cache Lines from RAM.
// ---------------------------------------------------------
public struct BadEnvelope
{
    public Guid TransactionId; // 16 bytes
    public int TenantId;       // 4 bytes
    public long AggregateId;   // 8 bytes
    public long Timestamp;     // 8 bytes
    public int PayloadLength;  // 4 bytes
    public long PayloadOffset; // 8 bytes
                               // Total = 48 bytes. But C# adds hidden padding, making it misaligned!
}

// ---------------------------------------------------------
// 2. THE VECTORVOLT WAY (Hardware Sympathy)
// We explicitly force the size to be exactly 64 bytes.
// The CPU swallows this perfectly in a single 1-nanosecond gulp.
// ---------------------------------------------------------
[StructLayout(LayoutKind.Explicit, Size = 64)]
public struct GoodEnvelope
{
    [FieldOffset(0)] public Guid TransactionId;
    [FieldOffset(16)] public int TenantId;
    [FieldOffset(20)] public long AggregateId;
    [FieldOffset(28)] public long Timestamp;
    [FieldOffset(36)] public int PayloadLength;
    [FieldOffset(40)] public long PayloadOffset;
    // Bytes 48 to 63 are safely padded with zeros to hit exactly 64.
}


// ---------------------------------------------------------
// 3. THE BENCHMARK
// We allocate an array of both, and see how fast the CPU processes them.
// ---------------------------------------------------------
[MemoryDiagnoser] // This tells us if we trigger the Garbage Collector
public class CacheLineBenchmark
{
    private BadEnvelope[] _badArray = new BadEnvelope[1_000_000];
    private GoodEnvelope[] _goodArray = new GoodEnvelope[1_000_000];

    [Benchmark(Baseline = true)]
    public int ProcessBadEnvelopes()
    {
        int total = 0;
        // The CPU chokes here because the data crosses the 64-byte boundary
        for (int i = 0; i < _badArray.Length; i++)
        {
            total += _badArray[i].TenantId;
        }
        return total;
    }

    [Benchmark]
    public int ProcessGoodEnvelopes()
    {
        int total = 0;
        // The CPU flies through this because every item is exactly 64 bytes
        for (int i = 0; i < _goodArray.Length; i++)
        {
            total += _goodArray[i].TenantId;
        }
        return total;
    }
}

public class Program
{
    public static void Main(string[] args)
    {
        //// Run this in RELEASE mode (not Debug)
        //var summary = BenchmarkRunner.Run<CacheLineBenchmark>();

        //// Run with Ctrl+F5 (Release Mode)
        //var gcsummary = BenchmarkRunner.Run<GcAllocationBenchmark>();

        //// Run with Ctrl+F5 (Release Mode)
        //var ringBuffersummary = BenchmarkRunner.Run<RingBufferBenchmark>();

        //// Run with Ctrl+F5 (Release Mode)
        //var prefetchSummary = BenchmarkRunner.Run<PrefetchBenchmark>();

        //// Run with Ctrl+F5 (Release Mode)
        //var simdScannerSummary = BenchmarkRunner.Run<SimdScannerBenchmark>();

        //// Run with Ctrl+F5 (Release Mode)
        //var commaHunterSummary = BenchmarkRunner.Run<CommaHunterBenchmark>();

        //// Run with Ctrl+F5 (Release Mode)
        //var memoryShufflerSummary = BenchmarkRunner.Run<MemoryShufflerBenchmark>();

        // This single line runs ALL benchmarks in the assembly
        // It will:
        // 1. Compile each method
        // 2. Warm up the JIT
        // 3. Run multiple iterations
        // 4. Calculate statistics
        // 5. Generate disassembly
        // 6. Output a beautiful table
        var sseAddSummary = BenchmarkRunner.Run<SseAddBenchmark>();

        // Optional: Verify correctness first
        var bench = new SseAddBenchmark();
        bench.Setup();

        Console.ReadLine();
    }
}