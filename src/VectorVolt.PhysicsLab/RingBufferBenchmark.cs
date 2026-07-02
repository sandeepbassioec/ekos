using BenchmarkDotNet.Attributes;
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

namespace VectorVolt.PhysicsLab
{
    [MemoryDiagnoser]
    public class RingBufferBenchmark
    {
        private const int Operations = 5_000_000; // 5 Million messages!

        // ---------------------------------------------------------
        // 1. THE STANDARD WAY (ConcurrentQueue)
        // Uses hidden locks, creates garbage on the heap, and forces Context Switches.
        // ---------------------------------------------------------
        private ConcurrentQueue<int> _standardQueue;

        // ---------------------------------------------------------
        // 2. THE VECTORVOLT WAY (Lock-Free Ring Buffer)
        // A simple array. We use padding to prevent "False Sharing" 
        // between the L1 caches of CPU Core 1 and CPU Core 2.
        // ---------------------------------------------------------
        private int[] _ringBuffer;
        private int _ringMask;

        // We force the sequence numbers to be 64 bytes apart so they don't 
        // accidentally sit on the same L1 Cache Line!
        [StructLayout(LayoutKind.Explicit, Size = 64)]
        public struct PaddedSequence
        {
            [FieldOffset(0)] public volatile int Value;
        }

        private PaddedSequence _writeSequence;
        private PaddedSequence _readSequence;

        [GlobalSetup]
        public void Setup()
        {
            _standardQueue = new ConcurrentQueue<int>();

            // Ring buffer size must be a power of 2 for fast bitwise masking
            int size = 1024 * 1024;
            _ringBuffer = new int[size];
            _ringMask = size - 1;

            _writeSequence.Value = 0;
            _readSequence.Value = 0;
        }

        [Benchmark(Baseline = true)]
        public Task ProcessWithStandardQueue()
        {
            // Thread A (Producer)
            var producer = Task.Run(() =>
            {
                for (int i = 0; i < Operations; i++)
                {
                    _standardQueue.Enqueue(i);
                }
            });

            // Thread B (Consumer)
            var consumer = Task.Run(() =>
            {
                int count = 0;
                while (count < Operations)
                {
                    if (_standardQueue.TryDequeue(out int item))
                    {
                        count++;
                    }
                }
            });

            return Task.WhenAll(producer, consumer);
        }

        [Benchmark]
        public Task ProcessWithLockFreeRing()
        {
            // Thread A (Writer)
            var producer = Task.Run(() =>
            {
                for (int i = 0; i < Operations; i++)
                {
                    // Calculate exactly where on the ring to put the data using super-fast bitwise math
                    int index = i & _ringMask;
                    _ringBuffer[index] = i;

                    // Tell the reader we moved forward! (Volatile bypasses the cache to ensure Thread B sees it)
                    _writeSequence.Value = i + 1;
                }
            });

            // Thread B (Reader/Chaser)
            var consumer = Task.Run(() =>
            {
                int count = 0;
                while (count < Operations)
                {
                    // Spin-wait: Do not go to sleep! Just wait for the writer to move forward.
                    while (_writeSequence.Value <= count)
                    {
                        Thread.SpinWait(1);
                    }

                    // Read the data
                    int index = count & _ringMask;
                    int item = _ringBuffer[index];
                    count++;
                }
            });

            return Task.WhenAll(producer, consumer);
        }
    }
}
