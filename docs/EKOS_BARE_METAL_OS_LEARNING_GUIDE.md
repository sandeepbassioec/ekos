# EKOS Bare-Metal OS: Complete Learning & Design Guide

**Version:** 1.0  
**Date:** 2026-07-12  
**Purpose:** Learn bootloader and kernel design from first principles

> **Goal:** Understand and build a custom OS where EKOS/VectorVolt runs directly on hardware with no Linux dependency. Learn how operating systems work by building one.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Part 1: Concepts & Fundamentals](#part-1-concepts--fundamentals)
3. [Part 2: Bootloader Design](#part-2-bootloader-design)
4. [Part 3: Kernel Architecture](#part-3-kernel-architecture)
5. [Part 4: Memory Management](#part-4-memory-management)
6. [Part 5: Interrupt & Exception Handling](#part-5-interrupt--exception-handling)
7. [Part 6: CPU Scheduling](#part-6-cpu-scheduling)
8. [Part 7: Hardware Abstraction Layer (HAL)](#part-7-hardware-abstraction-layer-hal)
9. [Part 8: Device Drivers (Minimal)](#part-8-device-drivers-minimal)
10. [Part 9: EKOS Integration](#part-9-ekos-integration)
11. [Part 10: Minimal Network Stack](#part-10-minimal-network-stack)
12. [Part 11: Build & Boot System](#part-11-build--boot-system)
13. [Implementation Roadmap](#implementation-roadmap)
14. [Learning Resources](#learning-resources)
15. [Appendix: Code Examples](#appendix-code-examples)

---

## Executive Summary

### What We're Building

A **bare-metal operating system** that:
- Boots directly on hardware (x86-64)
- Loads EKOS/VectorVolt as the primary application
- Provides minimal OS services (memory, scheduling, I/O)
- Achieves sub-microsecond latency (no OS preemption)
- Is completely self-contained (<50MB total)

### Why This Matters

**Traditional Stack:**
```
Hardware
  ↓
Linux Kernel (30MB)
  ↓
EKOS/VectorVolt (competes with OS for resources)
  ↓
Problems: Context switching, syscall overhead, kernel vulnerabilities
```

**Bare-Metal Stack:**
```
Hardware
  ↓
Custom Kernel (5MB, minimal)
  ↓
EKOS/VectorVolt (full hardware control, IS the OS)
  ↓
Benefits: Direct hardware access, guaranteed latency, bulletproof
```

### Key Learning Objectives

By the end of this guide, you'll understand:
1. ✅ How bootloaders work and what they do
2. ✅ How kernels initialize hardware
3. ✅ How memory management works (paging, virtual memory)
4. ✅ How interrupts and exceptions are handled
5. ✅ How CPU scheduling works
6. ✅ How to design a minimal operating system
7. ✅ How EKOS/VectorVolt can become the OS itself

---

## Part 1: Concepts & Fundamentals

### What Is an Operating System?

An OS is a **program that manages hardware resources** and provides services to applications.

**Key responsibilities:**
1. **Boot the hardware** → Initialize CPU, memory, devices
2. **Manage memory** → Virtual addressing, paging, protection
3. **Schedule CPU time** → Share CPU among processes
4. **Handle interrupts** → Respond to hardware events (disk, network)
5. **Provide I/O** → Abstract disk, network, peripherals
6. **Enforce security** → Isolate processes, protect memory

### Bootloader vs. Kernel

**Bootloader:**
```
Purpose: Get CPU and memory ready for kernel
Responsibilities:
  - Run in real mode (16-bit)
  - Load kernel into memory
  - Setup minimal structures
  - Jump to kernel entry point
Size: ~512 bytes to 64KB
Runs: Once at startup
```

**Kernel:**
```
Purpose: Manage hardware and provide OS services
Responsibilities:
  - Setup virtual memory
  - Initialize interrupt handlers
  - Start scheduler
  - Manage processes/threads
  - Handle device I/O
Size: 5-50MB
Runs: Always (until shutdown)
```

### x86-64 CPU Modes

**Real Mode (16-bit):**
```
- CPU starts here after power-on
- Can address 1MB of memory (DOS era)
- Bootloader runs in real mode
- Limited to 64KB segments
```

**Protected Mode (32-bit):**
```
- Can address 4GB of memory
- Memory protection via segments
- Better but outdated
- Not recommended for modern OS
```

**Long Mode (64-bit):**
```
- Can address 16 exabytes of memory
- Virtual memory (paging)
- Memory protection
- Modern standard
- Where kernel runs
```

**Bootloader Journey:**
```
Power on
  ↓
Real Mode (BIOS/UEFI loads bootloader)
  ↓
Bootloader: Enable A20 line (access past 1MB)
  ↓
Bootloader: Load kernel into memory
  ↓
Bootloader: Setup GDT (Global Descriptor Table)
  ↓
Bootloader: Switch to Protected Mode
  ↓
Bootloader: Setup paging, IDT (Interrupt Descriptor Table)
  ↓
Bootloader: Switch to Long Mode (64-bit)
  ↓
Bootloader: Jump to kernel entry point
  ↓
Kernel: Running in Long Mode
```

---

## Part 2: Bootloader Design

### Bootloader Stages

**Stage 1 (512 bytes - Master Boot Record):**
```
Location: First sector of disk (LBA 0)
Loaded by: BIOS/UEFI firmware

Responsibilities:
  - Initialize minimal stack
  - Check for BIOS extensions
  - Load Stage 2 from disk
  - Jump to Stage 2
```

**Stage 2 (64KB - Bootloader proper):**
```
Location: After MBR
Loaded by: Stage 1

Responsibilities:
  - Enable A20 line (if needed)
  - Load kernel binary from disk
  - Detect memory (INT 0x15)
  - Setup GDT (Global Descriptor Table)
  - Switch to Protected Mode
  - Setup paging (if needed)
  - Switch to Long Mode (64-bit)
  - Jump to kernel entry point
```

### Memory Layout at Boot Time

```
0x0000_0000 ┌──────────────────────┐
            │ IVT (Interrupt Vector)
            │ (256 interrupts)
            │ Loaded by BIOS
0x0000_0400 ├──────────────────────┤
            │ BIOS Data Area
            │ (memory size, disk info)
0x0000_7C00 ├──────────────────────┤
            │ Bootloader Stage 1    │ ← BIOS loads here
            │ (512 bytes MBR)       │
0x0000_7E00 ├──────────────────────┤
            │ (Free space)          │
0x0001_0000 ├──────────────────────┤
            │ Bootloader Stage 2    │ ← Stage 1 loads here
            │ (64KB)                │
0x0010_0000 ├──────────────────────┤
            │ Kernel Code & Data    │ ← Bootloader loads here
            │ (2MB)                 │
0x0030_0000 ├──────────────────────┤
            │ (Free for kernel)     │
            │ Paging tables, stacks,
            │ data structures
0x8000_0000 ├──────────────────────┤
            │ (Kernel VM starts here)
            │ (After paging enabled)
```

### Bootloader in Pseudocode

```pseudo
STAGE_1_BOOTLOADER:
  // Load at 0x7C00 by BIOS
  SETUP_STACK(0x7BFF)  // Stack grows down
  LOAD_STAGE_2(0x10000)
  JUMP_TO(0x10000)

STAGE_2_BOOTLOADER:
  // Load at 0x10000
  ENABLE_A20_LINE()  // Access past 1MB
  
  // Detect physical memory
  eax = 0xE820
  call INT_0x15  // Get memory map
  memory_regions = [...]
  
  // Load kernel from disk
  kernel_size = read_from_disk(KERNEL_LBA)
  kernel = load_into_memory(KERNEL_LBA, 0x100000)
  
  // Setup Global Descriptor Table (GDT)
  GDT[0] = null_descriptor
  GDT[1] = kernel_code_segment (64-bit, readable)
  GDT[2] = kernel_data_segment (64-bit, writable)
  GDT[3] = kernel_stack_segment
  gdtr.base = &GDT[0]
  gdtr.limit = sizeof(GDT)
  lgdt gdtr  // Load GDT register
  
  // Switch to Protected Mode
  cr0 |= 0x1  // Set PE bit (Protection Enable)
  jmp FAR 0x8:setup_paging  // Jump to Protected Mode code
  
  setup_paging:
    // Setup paging tables (if needed)
    // Create page directory
    // Create page tables
    // Map kernel identity (VA = PA)
    cr3 = page_directory_addr
    cr0 |= 0x80000000  // Set PG bit (paging enable)
    
    // Setup EFER register for Long Mode
    msr = 0xC0000080  // EFER
    rdmsr
    eax |= 0x100  // Set LME bit (Long Mode Enable)
    wrmsr
    
    // Enable PAE (Physical Address Extension)
    cr4 |= 0x20  // Set PAE bit
    
    // Switch to Long Mode
    cr0 |= 0x80000001  // PE + PG bits
    jmp FAR 0x18:kernel_entry  // Jump to Long Mode
    
  kernel_entry:
    // 64-bit kernel code (loaded at 0x100000)
    // Kernel handles rest of initialization
    jmp kernel_main()
```

### What the Bootloader Sets Up

```
After bootloader, before kernel:

✓ CPU in Long Mode (64-bit)
✓ Virtual memory enabled (paging)
✓ Global Descriptor Table loaded
✓ Memory map available (INT 0x15 results)
✓ Kernel loaded at 0x100000
✓ Stack ready
✓ No interrupts yet (IDT not set up)

Kernel's job: Complete the setup
```

---

## Part 3: Kernel Architecture

### Kernel Components

```
Kernel (runs in privileged mode)
├── Bootstrap Code
│   ├── Initialize CPU
│   ├── Setup IDT (Interrupt Descriptor Table)
│   ├── Setup memory
│   └── Start main kernel
├── Memory Management
│   ├── Virtual memory
│   ├── Paging
│   ├── Heap allocator
│   └── Memory protection
├── Interrupt Handler
│   ├── CPU exceptions (divide by zero, page fault)
│   ├── Hardware interrupts (disk, network, timer)
│   └── Software interrupts (syscalls)
├── Scheduler
│   ├── Process/thread management
│   ├── CPU time allocation
│   └── Context switching
├── Hardware Abstraction Layer (HAL)
│   ├── CPU operations
│   ├── I/O operations
│   └── Device management
├── Device Drivers
│   ├── Disk controller
│   ├── Network interface
│   ├── Serial console
│   └── Timer
└── EKOS/VectorVolt Runtime
    ├── EKOS Core
    ├── VectorVolt Engine
    └── Application Logic
```

### Kernel Memory Layout (Virtual Addressing)

```
Virtual Address Space (64-bit):

0x0000_0000_0000_0000 ┌──────────────────────┐
                      │ User Space           │
                      │ (if we support apps) │
                      │ Usually not used     │
0x0000_FFFF_FFFF_FFFF ├──────────────────────┤
                      │ Kernel Space         │
                      │ (higher half)        │
0xFFFF_8000_0000_0000 ├──────────────────────┤
                      │ Kernel Code & Data   │
                      │ (linked here)        │
0xFFFF_FFFF_FFFF_FFFF └──────────────────────┘

Kernel typically maps:
  - Code: executable, read-only
  - Data: read-write
  - Heap: growable, read-write
  - Stack: per-thread, read-write
  - Device memory: mapped via MMIO
```

### Kernel Initialization Pseudocode

```pseudo
KERNEL_ENTRY:
  // Called by bootloader in 64-bit mode
  // Identity mapped (VA = PA for now)
  
  // 1. Setup stack for kernel
  rsp = kernel_stack_top
  
  // 2. Setup Interrupt Descriptor Table (IDT)
  for i in 0..255:
    IDT[i] = setup_interrupt_handler(i)
  lidt idt_register
  
  // 3. Setup memory management
  heap_init()
  virtual_memory_init()
  paging_setup()
  
  // 4. Setup timer interrupt (preemption)
  timer_init(1000)  // 1000 Hz = 1ms timer tick
  sti()  // Enable interrupts
  
  // 5. Setup scheduler
  scheduler_init()
  
  // 6. Start EKOS as first "process"
  ekos_process = create_process("ekos", ekos_main)
  scheduler_add(ekos_process)
  
  // 7. Start scheduler (infinite loop)
  scheduler_run()
  
  // Never returns
```

### Kernel Privilege Levels

```
Ring 0: Kernel (full hardware access)
  - Can execute privileged instructions (lgdt, lidt, cr0 write)
  - Can access all memory
  - Can configure interrupts
  - Can access device I/O

Ring 1-2: (usually unused)

Ring 3: User (restricted access)
  - Cannot execute privileged instructions
  - Memory protection (only mapped pages)
  - Cannot change interrupt handlers
  - Cannot access devices directly (must use syscalls)

For EKOS:
  - Kernel runs in Ring 0
  - EKOS runs in Ring 0 (trusted)
  - Can't have unprivileged apps anyway
```

---

## Part 4: Memory Management

### Virtual Memory Concept

**Why Virtual Memory?**

```
Without virtual memory:
  - Program A: accesses address 0x1000
  - Program B: accesses address 0x1000
  - Both access same physical memory (crash!)

With virtual memory:
  - Program A: virtual address 0x1000 → physical 0x10000
  - Program B: virtual address 0x1000 → physical 0x20000
  - Each sees separate memory (safe!)
```

### Paging

**How it works:**

```
Virtual Address: 0xFFFF_8000_0001_2345

Breakdown (x86-64):
  Bits 63-48: Sign extension (all 1s for kernel)
  Bits 47-39: PML4 index (table 1)
  Bits 38-30: PDPT index (table 2)
  Bits 29-21: PD index (table 3)
  Bits 20-12: PT index (page table)
  Bits 11-0:  Page offset (4KB pages)

Translation:
  1. Load CR3 (page directory base)
  2. Look up PML4[1] → points to PDPT
  3. Look up PDPT[2] → points to PD
  4. Look up PD[3] → points to PT
  5. Look up PT[4096] → points to physical page
  6. Add page offset (0x345)
  7. Final physical address: 0x<physical_page> + 0x345
```

**Memory layout with paging:**

```
Virtual Memory                          Physical Memory
────────────────────                    ─────────────────

0xFFFF_8000_0000_0000
    ↓ (PML4, PDPT, PD, PT tables)
    ↓
Physical address 0x100_0000
    ↓
0xFFFF_8000_0001_0000
    ↓ (PML4, PDPT, PD, PT tables)
    ↓
Physical address 0x200_0000
    ↓
0xFFFF_8000_0002_0000
    ↓
Physical address 0x300_0000
    ↓
...
```

### Memory Allocator (Heap)

**Simple bump allocator:**

```pseudo
MEMORY_ALLOCATOR:
  heap_base = 0xFFFF_8000_1000_0000
  heap_ptr = heap_base
  
  FUNCTION allocate(size):
    if heap_ptr + size > heap_end:
      return NULL  // Out of memory
    
    ptr = heap_ptr
    heap_ptr += size
    return ptr
  
  // No freeing (for simplicity)
  // Use Rust's Box<T> for cleanup
```

**Better allocator (buddy system):**

```
Divide heap into powers of 2:
  64 bytes:   free ███ ███
  128 bytes:  free
  256 bytes:  free ███
  512 bytes:  allocated
  1024 bytes: free

When requesting 128 bytes:
  - Check 128-byte list: empty
  - Take 256-byte block: split into two 128s
  - Return one 128-byte block
  - Keep other for future allocations

When freeing:
  - Return block to appropriate list
  - Coalesce with buddy if both free
```

### Page Protection

```
Each page table entry has flags:

Bit 0: Present (in memory?)
Bit 1: Read/Write (writable?)
Bit 2: User/Supervisor (user-accessible?)
Bit 3-4: Cache settings
Bit 5-11: Custom flags
Bit 63: No-execute (NX bit)

Examples:
  Kernel code: Present=1, RW=0, US=0, NX=0 (exec, read-only)
  Kernel data: Present=1, RW=1, US=0, NX=1 (no-exec, read-write)
  Stack:       Present=1, RW=1, US=0, NX=1 (no-exec, read-write)
  MMIO:        Present=1, RW=1, US=0, Cache=0 (no-cache)
```

---

## Part 5: Interrupt & Exception Handling

### Interrupts vs. Exceptions

**Exceptions (from CPU):**
```
Synchronous (happen during instruction execution)
Examples:
  - Divide by zero (DIV 0)
  - Page fault (memory not present)
  - General protection fault
  - Stack overflow
  - Breakpoint (INT 3)
```

**Interrupts (from hardware):**
```
Asynchronous (can happen anytime)
Examples:
  - Timer tick (every 1ms)
  - Disk finished I/O
  - Network packet arrived
  - Keyboard key pressed
  - Serial port data available
```

### Interrupt Descriptor Table (IDT)

**IDT structure:**

```
IDT[256 entries] (one per interrupt/exception)

Entry format:
  Bytes 0-1:   Handler address (bits 0-15)
  Bytes 2-3:   Code segment selector
  Bytes 4-5:   Flags (present, DPL, type)
  Bytes 6-7:   Handler address (bits 16-31)
  Bytes 8-11:  Handler address (bits 32-63)
  Bytes 12-15: Reserved

Example IDT setup:

IDT[0] = {
  handler: divide_by_zero_handler,
  segment: KERNEL_CODE_SEGMENT,
  present: 1,
  dpl: 0,  // Ring 0 (kernel only)
  type: TRAP_GATE
}

IDT[8] = {
  handler: timer_handler,
  segment: KERNEL_CODE_SEGMENT,
  present: 1,
  dpl: 0,
  type: INTERRUPT_GATE
}

IDT[32] = {
  handler: disk_interrupt_handler,
  segment: KERNEL_CODE_SEGMENT,
  present: 1,
  dpl: 0,
  type: INTERRUPT_GATE
}
```

### Interrupt Handler Flow

```
Hardware generates interrupt
  ↓
CPU:
  1. Disable further interrupts (CLI)
  2. Save registers (push to kernel stack)
  3. Look up IDT[interrupt_number]
  4. Jump to handler code
  ↓
Handler code (assembly):
  1. Save all registers (RAX, RCX, RDX, etc.)
  2. Load kernel data segment
  3. Call C handler function
  4. Restore registers
  5. Return from interrupt (IRET)
  ↓
CPU:
  1. Restore stack pointer
  2. Restore flags (includes interrupt flag)
  3. Resume previous code

Example handler (assembly):

timer_interrupt_handler:
  push rax
  push rcx
  push rdx
  mov rdi, rsp  // Pass register context to C function
  call timer_interrupt_c_handler
  pop rdx
  pop rcx
  pop rax
  iret  // Return from interrupt
```

### Exception Handling

**Page Fault Example:**

```
Scenario: Kernel accesses address that's not mapped

CPU:
  1. Generates exception #14 (page fault)
  2. Saves error code (what went wrong)
  3. Calls IDT[14] → page_fault_handler
  ↓
page_fault_handler:
  error_code = error from CPU
  faulting_addr = CR2 register (the address that faulted)
  
  if error_code & 0x1 == 0:
    // Page not present
    if faulting_addr in [heap_start, heap_end]:
      // Growing heap
      allocate_page(faulting_addr)
      add_page_to_pagetable(faulting_addr)
      return  // Retry instruction
    else:
      // Invalid access
      PANIC("Kernel page fault at", faulting_addr)
  
  else if error_code & 0x2 != 0:
    // Read-only page
    PANIC("Write to read-only page", faulting_addr)
  
  return  // Resume execution
```

---

## Part 6: CPU Scheduling

### Why Scheduling?

**Problem:** Multiple tasks, one CPU

```
Without scheduler:
  EKOS task runs forever
  VectorVolt never gets CPU time
  System hangs

With scheduler:
  Timer interrupt every 1ms
  Scheduler switches between tasks
  All tasks get CPU time fairly
```

### Round-Robin Scheduler

**Simplest approach:**

```
Structure:

Process {
  id: u32,
  name: String,
  state: RUNNING | READY | BLOCKED,
  priority: 0-255 (0=highest),
  cpu_time_slice: 10ms,
  registers: [rax, rbx, rcx, ...],
  page_table: *PDPT,
}

ready_queue = [
  ekos_process,
  vectorvolt_process,
]

SCHEDULER:

current_process = ready_queue[0]

timer_interrupt_handler():
  // Called every 1ms
  if current_process.time_slice > 0:
    current_process.time_slice -= 1ms
    return  // Let process continue
  
  // Time slice expired, switch
  current_process.time_slice = 10ms
  current_process = ready_queue.next()
  
  // Save old process context
  save_registers(current_process)
  
  // Load new process context
  load_registers(new_process)
  cr3 = new_process.page_table
  
  // Resume new process (iret will restore CPU state)

schedule_process(process):
  if process.state == READY:
    ready_queue.push(process)

// Infinite loop in main thread
scheduler_main():
  while true:
    current_process = pick_next_from_queue()
    switch_to_process(current_process)
    // Never returns (scheduler_run handles returns)
```

### Context Switching

**What gets saved:**

```
When switching from Process A to Process B:

Save Process A:
  RAX, RBX, RCX, RDX, RSI, RDI
  R8-R15
  RBP (frame pointer)
  RSP (stack pointer)
  RIP (instruction pointer)
  RFLAGS (flags)
  CR3 (page table)
  
Restore Process B:
  Load all the above for Process B
  Load CR3 (changes virtual memory)
  Execute IRET to jump to Process B's RIP
```

---

## Part 7: Hardware Abstraction Layer (HAL)

### Why HAL?

Keep hardware-specific code separate from kernel logic.

```
Bad (hardware mixed with logic):
  if (cpu_model == INTEL) {
    // Intel-specific code
  } else if (cpu_model == AMD) {
    // AMD-specific code
  }
  // Logic mixed with hardware details

Good (HAL abstraction):
  hal_read_memory(addr)  // Same everywhere
  hal_write_memory(addr, value)
  // Hardware hidden
```

### HAL Functions

**CPU operations:**

```rust
pub mod hal {
    // CPU control
    pub fn read_cr0() -> u64 { ... }
    pub fn write_cr0(val: u64) { ... }
    pub fn read_cr3() -> u64 { ... }
    pub fn write_cr3(val: u64) { ... }
    pub fn read_rflags() -> u64 { ... }
    pub fn enable_interrupts() { asm!("sti") }
    pub fn disable_interrupts() { asm!("cli") }
    
    // MSR (Model Specific Register)
    pub fn read_msr(msr: u32) -> u64 { ... }
    pub fn write_msr(msr: u32, val: u64) { ... }
    
    // GDT/IDT
    pub fn load_gdt(gdt_ptr: &GdtRegister) { ... }
    pub fn load_idt(idt_ptr: &IdtRegister) { ... }
    
    // Memory barriers
    pub fn memory_fence() { asm!("mfence") }
}

pub mod io {
    // I/O port operations
    pub fn inb(port: u16) -> u8 { ... }
    pub fn outb(port: u16, val: u8) { ... }
    pub fn inw(port: u16) -> u16 { ... }
    pub fn outw(port: u16, val: u16) { ... }
}
```

**Memory operations:**

```rust
pub fn read_memory(addr: u64) -> u64 {
    unsafe { *(addr as *const u64) }
}

pub fn write_memory(addr: u64, val: u64) {
    unsafe { *(addr as *mut u64) = val }
}

pub fn copy_memory(src: u64, dst: u64, size: u64) {
    for i in 0..size {
        let byte = read_memory(src + i);
        write_memory(dst + i, byte);
    }
}
```

---

## Part 8: Device Drivers (Minimal)

### Which Drivers Do We Need?

For EKOS OS, minimal:

```
✓ Serial Port (console output)
  - Simple, 16-byte FIFO
  - Good for debugging
  
✓ Timer (scheduling)
  - Local APIC timer or PIT
  - Generates interrupts
  
✓ Disk Controller (load kernel, store data)
  - SATA or IDE
  - Read sectors
  
? Network (optional)
  - If we want networking
  - Ethernet NIC
  
? Graphics (optional)
  - Probably not for EKOS
  - VGA text mode is enough
```

### Serial Port Driver Example

**Hardware registers:**

```
Serial port (COM1) at I/O port 0x3F8

Register layout:
  0x3F8: Data (read/write)
  0x3F9: Interrupt enable
  0x3FA: FIFO control
  0x3FB: Line control
  0x3FC: Modem control
  0x3FD: Line status
  0x3FE: Modem status

Initialization:
  1. Set baud rate (divisor = 115200 / desired_baud)
  2. Set line control (8 bits, no parity, 1 stop bit)
  3. Enable FIFO
  4. Send character: write to 0x3F8
  5. Receive character: read from 0x3F8
```

**Driver code:**

```rust
pub struct SerialPort {
    port: u16,
}

impl SerialPort {
    pub fn new(port: u16) -> Self {
        SerialPort { port }
    }
    
    pub fn init(&self) {
        // Set baud rate to 115200
        let divisor = 1;  // 115200 / 115200
        
        // Enable DLAB (divisor latch access bit)
        outb(self.port + 3, 0x80);
        
        // Set divisor
        outb(self.port + 0, divisor & 0xFF);
        outb(self.port + 1, (divisor >> 8) & 0xFF);
        
        // Disable DLAB, set 8 bits, no parity, 1 stop bit
        outb(self.port + 3, 0x03);
        
        // Enable FIFO
        outb(self.port + 2, 0xC7);
        
        // Enable interrupts
        outb(self.port + 1, 0x01);
    }
    
    pub fn write(&self, byte: u8) {
        // Wait for transmit buffer empty
        while (inb(self.port + 5) & 0x20) == 0 {}
        
        // Write byte
        outb(self.port + 0, byte);
    }
    
    pub fn read(&self) -> u8 {
        // Wait for data available
        while (inb(self.port + 5) & 0x01) == 0 {}
        
        // Read byte
        inb(self.port + 0)
    }
    
    pub fn write_string(&self, s: &str) {
        for byte in s.bytes() {
            self.write(byte);
        }
    }
}

// Usage
let serial = SerialPort::new(0x3F8);
serial.init();
serial.write_string("Hello from EKOS OS!\n");
```

### Timer Driver (for scheduling)

**Local APIC Timer:**

```rust
pub struct Timer;

impl Timer {
    pub fn init(frequency_hz: u32) {
        // Set timer interrupt vector
        let timer_vector = 32;  // First available vector
        
        // Configure Local APIC timer
        // LVT Timer register at 0xFEE00300
        let lvt_timer = 0xFEE00300 as *mut u32;
        
        // Vector = 32, timer mode = periodic
        unsafe {
            *lvt_timer = (timer_vector & 0xFF) | (1 << 17);
        }
        
        // Set timer divisor (APIC bus clock / 16)
        let timer_div = 0xFEE00380 as *mut u32;
        unsafe {
            *timer_div = 0x03;  // Divisor = 16
        }
        
        // Set initial count (APIC frequency / desired frequency)
        let apic_freq = 1_000_000_000;  // Example: 1 GHz APIC clock
        let initial_count = apic_freq / (16 * frequency_hz);
        
        let timer_init = 0xFEE00380 as *mut u32;
        unsafe {
            *timer_init = initial_count;
        }
    }
    
    pub fn handler() {
        // Called every timer tick
        scheduler_tick();
        
        // Send EOI (End Of Interrupt) to APIC
        let eoi = 0xFEE000B0 as *mut u32;
        unsafe {
            *eoi = 0;
        }
    }
}

// Timer interrupt handler
fn scheduler_tick() {
    // Decrement process time slice
    // If expired, switch to next process
    SCHEDULER.tick();
}
```

---

## Part 9: EKOS Integration

### EKOS as the Primary Application

**Architecture:**

```
Boot sequence:

Bootloader
  ↓
Kernel initialization
  ↓
Setup memory, interrupts, scheduler
  ↓
Create EKOS process
  ↓
Start scheduler (EKOS runs)
  ↓
EKOS initialization
  ↓
Initialize VectorVolt
  ↓
VectorVolt POD Manager
  ↓
Listen for network requests
  ↓
EKOS/VectorVolt running
```

**Kernel provides EKOS with:**

```
1. Memory allocation
   ekos_malloc() → calls kernel allocator
   ekos_free() → calls kernel deallocator

2. Process management
   ekos_create_pod() → kernel creates new process
   ekos_kill_pod() → kernel terminates process
   ekos_get_cpu_time() → total CPU time used

3. Interrupts
   ekos_register_interrupt_handler() → custom handler
   ekos_enable_interrupt() → enable specific IRQ
   
4. I/O
   ekos_read_disk() → kernel reads sector
   ekos_write_disk() → kernel writes sector
   ekos_network_send() → kernel sends packet
   ekos_network_recv() → kernel receives packet

5. Scheduling
   ekos_set_priority() → adjust process priority
   ekos_get_cpu_usage() → current CPU %
   ekos_wait_for_event() → block until event
```

**EKOS syscall interface:**

```rust
pub enum Syscall {
    // Memory
    Malloc { size: u64 },
    Free { ptr: u64 },
    
    // Process
    CreateProcess { name: String, entry: u64 },
    KillProcess { pid: u32 },
    GetCpuTime { pid: u32 },
    SetPriority { pid: u32, priority: u8 },
    
    // I/O
    ReadDisk { lba: u64, count: u32 },
    WriteDisk { lba: u64, data: &[u8] },
    SendNetwork { data: &[u8] },
    RecvNetwork { timeout_ms: u32 },
    
    // Interrupts
    RegisterHandler { irq: u32, handler: fn() },
    EnableInterrupt { irq: u32 },
    DisableInterrupt { irq: u32 },
}

// In EKOS
pub fn syscall(syscall: Syscall) -> Result {
    // Call into kernel
    kernel_handle_syscall(syscall)
}
```

---

## Part 10: Minimal Network Stack

### Networking Layers

```
EKOS Application Layer
  ↓
TCP/IP (kernel)
  ├─ TCP (reliable streams)
  ├─ UDP (unreliable datagrams)
  ├─ IP (routing)
  └─ Ethernet (MAC addresses)
  ↓
Network Driver (kernel)
  ├─ Ethernet NIC driver
  └─ Interrupt handling
  ↓
Hardware (NIC)
  ├─ RX ring buffer
  ├─ TX ring buffer
  └─ DMA to/from memory
```

### Minimal TCP/IP Stack

**Ethernet frame:**

```
Destination MAC (6 bytes)
Source MAC (6 bytes)
Type (2 bytes) [0x0800 = IPv4]
Data (payload)
CRC (4 bytes)
```

**IPv4 packet:**

```
Version/IHL (1 byte)
Type of Service (1 byte)
Total Length (2 bytes)
Identification (2 bytes)
Flags/Fragment Offset (2 bytes)
TTL (1 byte)
Protocol (1 byte) [6 = TCP, 17 = UDP]
Checksum (2 bytes)
Source IP (4 bytes)
Destination IP (4 bytes)
Options/Data (variable)
```

**TCP segment:**

```
Source Port (2 bytes)
Destination Port (2 bytes)
Sequence Number (4 bytes)
Acknowledgement (4 bytes)
Flags (1 byte) [SYN, ACK, FIN, RST]
Window Size (2 bytes)
Checksum (2 bytes)
Urgent Pointer (2 bytes)
Options/Data (variable)
```

**Minimal kernel network handler:**

```rust
pub struct NetworkStack {
    mac_address: [u8; 6],
    ip_address: [u8; 4],
    routing_table: Vec<Route>,
}

impl NetworkStack {
    pub fn init(mac: [u8; 6], ip: [u8; 4]) -> Self {
        NetworkStack {
            mac_address: mac,
            ip_address: ip,
            routing_table: vec![],
        }
    }
    
    pub fn on_ethernet_frame(&mut self, frame: &[u8]) {
        // Parse ethernet
        let src_mac = &frame[6..12];
        let dst_mac = &frame[0..6];
        let ethertype = u16::from_be_bytes([frame[12], frame[13]]);
        let payload = &frame[14..];
        
        match ethertype {
            0x0800 => self.on_ipv4_packet(src_mac, payload),
            0x0806 => self.on_arp_packet(src_mac, payload),
            _ => {}  // Unknown
        }
    }
    
    pub fn on_ipv4_packet(&mut self, src_mac: &[u8], packet: &[u8]) {
        let protocol = packet[9];
        let src_ip = &packet[12..16];
        let dst_ip = &packet[16..20];
        let payload = &packet[20..];
        
        // Check if packet is for us
        if dst_ip != self.ip_address.as_ref() {
            return;  // Not for us
        }
        
        match protocol {
            6 => self.on_tcp_segment(src_ip, src_mac, payload),
            17 => self.on_udp_packet(src_ip, src_mac, payload),
            _ => {}
        }
    }
    
    pub fn on_tcp_segment(&mut self, src_ip: &[u8], src_mac: &[u8], segment: &[u8]) {
        let src_port = u16::from_be_bytes([segment[0], segment[1]]);
        let dst_port = u16::from_be_bytes([segment[2], segment[3]]);
        let flags = segment[13];
        let data = &segment[20..];
        
        // Forward to EKOS TCP handler
        EKOS_TCP_HANDLER.handle_segment(src_ip, src_port, dst_port, flags, data);
    }
}

// Send packet from EKOS
pub fn send_ethernet_frame(dst_mac: &[u8], ethertype: u16, payload: &[u8]) {
    let mut frame = Vec::new();
    
    // Build ethernet frame
    frame.extend_from_slice(dst_mac);
    frame.extend_from_slice(&NETWORK_STACK.mac_address);
    frame.extend_from_slice(&ethertype.to_be_bytes());
    frame.extend_from_slice(payload);
    
    // Send via NIC driver
    NIC_DRIVER.send(&frame);
}
```

---

## Part 11: Build & Boot System

### Build Process

**Bootloader compilation:**

```bash
# Boot sector (Stage 1)
nasm -f bin bootloader/boot.asm -o bootloader/boot.bin

# Stage 2 bootloader
nasm -f elf64 bootloader/bootloader.asm -o bootloader/bootloader.o
ld -T bootloader/link.ld bootloader/bootloader.o -o bootloader/bootloader.bin
```

**Kernel compilation:**

```bash
# Rust kernel
cargo build --release --target x86_64-ekos-osdev

# Convert to binary
objcopy -O binary target/x86_64-ekos-osdev/release/kernel kernel.bin
```

**Disk image creation:**

```bash
# Create disk image
dd if=/dev/zero of=disk.img bs=1M count=100

# Write bootloader
dd if=bootloader/boot.bin of=disk.img bs=512 count=1 conv=notrunc

# Write Stage 2 bootloader
dd if=bootloader/bootloader.bin of=disk.img bs=512 seek=1 conv=notrunc

# Write kernel
dd if=kernel.bin of=disk.img bs=512 seek=100 conv=notrunc

# Write EKOS/VectorVolt
dd if=ekos.bin of=disk.img bs=512 seek=10000 conv=notrunc
```

**Boot testing:**

```bash
# QEMU
qemu-system-x86_64 -drive format=raw,file=disk.img -serial stdio

# Real hardware
# Burn to USB: dd if=disk.img of=/dev/sdX
# Boot from USB
```

### Grub2 Alternative (Simpler)

Instead of writing bootloader from scratch, use GRUB2:

```bash
# grub.cfg
menuentry "EKOS" {
    multiboot /kernel.bin
    module /ekos.bin
    boot
}

# Build GRUB image
grub-mkimage -O i386-pc -o boot.img -c grub.cfg \
    biosdisk part_msdos ext2

# Create disk
dd if=/dev/zero of=disk.img bs=1M count=100
dd if=boot.img of=disk.img conv=notrunc

# Install GRUB
grub-install --no-floppy --root-directory=/ disk.img
```

---

## Implementation Roadmap

### Phase 1: Bootloader (2-4 weeks)

```
Goal: Boot to kernel entry point

Tasks:
  1. Write boot.asm (Stage 1, 512 bytes)
     - Initialize stack
     - Load Stage 2
  
  2. Write bootloader.asm (Stage 2)
     - Detect memory
     - Setup GDT
     - Enable A20
     - Switch to Protected Mode
     - Setup paging
     - Switch to Long Mode
     - Jump to kernel
  
  3. Test in QEMU
     - Verify kernel entry point reached
     - Check memory map
  
  4. Debug tools
     - Serial port output
     - Breakpoint support
```

### Phase 2: Kernel Core (3-5 weeks)

```
Goal: Kernel running, interrupts working

Tasks:
  1. Kernel entry point (_start())
     - Setup kernel stack
     - Call kernel_main()
  
  2. Interrupt handling
     - Setup IDT
     - Exception handlers (page fault, divide by zero)
     - Hardware interrupt handlers
  
  3. Serial port driver
     - Output for debugging
     - printf() implementation
  
  4. Memory management
     - Virtual memory setup
     - Heap allocator
     - Page allocation
  
  5. Test
     - Trigger exceptions
     - Verify handlers work
```

### Phase 3: Scheduler (2-3 weeks)

```
Goal: Multiple processes, context switching

Tasks:
  1. Process structure
     - Process control block
     - Register saving
     - Page table per process
  
  2. Timer driver
     - Setup periodic interrupts (1kHz)
     - Measure scheduling time
  
  3. Scheduler implementation
     - Ready queue
     - Context switching
     - Round-robin scheduling
  
  4. Test
     - Create multiple processes
     - Verify CPU time sharing
```

### Phase 4: Device Drivers (2-3 weeks)

```
Goal: Disk and network I/O

Tasks:
  1. Disk driver (SATA/IDE)
     - Read sectors
     - Write sectors
     - Interrupt-driven I/O
  
  2. Network driver (Ethernet NIC)
     - Receive packets
     - Send packets
     - DMA setup
  
  3. Simple TCP/IP stack
     - Ethernet framing
     - IP routing
     - TCP/UDP sockets
  
  4. Test
     - Read kernel from disk
     - Send/receive packets
```

### Phase 5: EKOS Integration (3-4 weeks)

```
Goal: EKOS/VectorVolt running on EKOS OS

Tasks:
  1. Syscall interface
     - EKOS calls kernel for services
  
  2. POD Manager integration
     - Create PODs as kernel processes
     - Resource limits per POD
  
  3. Memory management for EKOS
     - Allocate memory for widgets
     - Protect memory regions
  
  4. EKOS runtime
     - Load EKOS into kernel
     - Initialize VectorVolt
     - Start POD Manager
  
  5. Test
     - EKOS generates features
     - Features run as PODs
     - Features can query VectorVolt
```

**Total: ~12-18 weeks (3-4 months)**

---

## Learning Resources

### Books & References

**OS Fundamentals:**
- "Operating Systems: Three Easy Pieces" (free online)
- "Modern Operating Systems" by Tanenbaum
- "Intel 64 and IA-32 Architectures Software Developer Manual" (official)

**x86-64 Assembly:**
- "Introduction to x86-64 Assembly" (online guides)
- "x86-64 Architecture Cheat Sheet"
- Intel SDM (Software Developer Manual) - official reference

**Bootloaders:**
- "The Art of Computer Systems Performance Analysis"
- OSDev.org bootloader tutorials
- GRUB documentation

**Kernels:**
- Linux kernel source (read specific parts)
- xv6 operating system (educational)
- OSDev.org kernel tutorials
- Raspberry Pi kernel (simpler)

### Practice Environment

**QEMU (virtual machine):**
```bash
# Install
apt-get install qemu-system-x86

# Boot disk image
qemu-system-x86_64 -drive format=raw,file=disk.img -serial stdio

# Debug with GDB
qemu-system-x86_64 -s -S -drive format=raw,file=disk.img &
gdb
(gdb) target remote localhost:1234
(gdb) symbol-file kernel.elf
(gdb) break kernel_main
(gdb) continue
```

**Real Hardware (later):**
- USB stick boot
- Serial cable for debugging
- JTAG debugger (advanced)

### Development Environment

**Rust for kernel:**
```toml
# Cargo.toml
[package]
name = "ekos-kernel"
version = "0.1.0"
edition = "2021"

[dependencies]
x86_64 = "0.14"
lazy_static = "1.4"

[[bin]]
name = "kernel"
path = "src/main.rs"

[profile.release]
opt-level = 3
lto = true
```

**Target specification:**

```json
{
  "llvm-target": "x86_64-unknown-none",
  "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128",
  "arch": "x86_64",
  "target-os": "none",
  "target-env": "gnu",
  "disable-redzone": true,
  "features": "-mmx,-sse,+soft-float",
  "code-model": "kernel",
  "relocation-model": "static",
  "panic": "abort"
}
```

---

## Appendix: Code Examples

### A. Boot.asm (Stage 1 - x86-64)

```asm
[BITS 16]
[ORG 0x7C00]

BOOT:
    cli                    ; Disable interrupts
    mov ax, 0x0000        ; Setup stack
    mov ss, ax
    mov sp, 0x7C00
    
    mov ax, 0x0000        ; Setup data segment
    mov ds, ax
    mov es, ax
    
    ; Load Stage 2 from disk
    mov ah, 0x02          ; Read disk
    mov al, 4             ; Read 4 sectors
    mov ch, 0             ; Cylinder 0
    mov cl, 2             ; Sector 2
    mov dh, 0             ; Head 0
    mov dl, 0x80          ; Drive 0 (first HDD)
    mov bx, 0x1000        ; Load to 0x1000
    int 0x13
    
    jmp 0x1000            ; Jump to Stage 2

times 510-($-$$) db 0
dw 0xAA55               ; Boot sector signature
```

### B. Kernel Entry (Rust)

```rust
// src/main.rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Kernel initialization
    
    // Setup serial port
    serial::init();
    println!("EKOS OS Booting...");
    
    // Setup memory
    memory::init();
    println!("Memory initialized");
    
    // Setup interrupts
    interrupts::init();
    println!("Interrupts initialized");
    
    // Setup scheduler
    scheduler::init();
    println!("Scheduler initialized");
    
    // Create EKOS process
    let ekos = process::create("ekos", ekos_main);
    scheduler::add(ekos);
    
    // Start scheduler
    scheduler::run();  // Never returns
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {}
}
```

### C. Interrupt Handler (Rust)

```rust
// src/interrupts.rs
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct InterruptFrame {
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let addr = x86_64::registers::control::Cr2::read();
    
    if is_kernel_memory(addr) {
        if is_stack_growth(addr) {
            allocate_page(addr);
        } else {
            println!("KERNEL PAGE FAULT: 0x{:x}", addr);
            panic!("Page fault");
        }
    } else {
        println!("USER PAGE FAULT: 0x{:x}", addr);
    }
}
```

### D. Memory Allocator (Rust)

```rust
// src/memory.rs
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
}

impl BumpAllocator {
    pub fn new(heap_start: usize, heap_size: usize) -> Self {
        BumpAllocator {
            heap_start,
            heap_end: heap_start + heap_size,
            next: heap_start,
        }
    }
    
    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        let alloc_start = self.next;
        self.next += size;
        
        if self.next > self.heap_end {
            self.next -= size;
            return None;
        }
        
        Some(alloc_start as *mut u8)
    }
}

// Global allocator
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(
    BumpAllocator::new(HEAP_START, HEAP_SIZE)
);
```

---

## Summary

You now understand:

1. ✅ **Bootloader design** - How to initialize hardware and load kernel
2. ✅ **Kernel architecture** - Core components and responsibilities
3. ✅ **Memory management** - Virtual memory, paging, protection
4. ✅ **Interrupt handling** - CPU exceptions and hardware interrupts
5. ✅ **Process scheduling** - Context switching, fairness
6. ✅ **Hardware abstraction** - Keeping hardware details separate
7. ✅ **Device drivers** - Serial, timer, disk, network
8. ✅ **EKOS integration** - Running EKOS as the primary OS application
9. ✅ **Networking** - Minimal TCP/IP stack for communication
10. ✅ **Implementation roadmap** - 12-18 week plan to build it

---

**Next Steps:**

1. **Study the concepts** - Read "Operating Systems: Three Easy Pieces" (free)
2. **Setup QEMU** - Practice compiling and booting simple kernels
3. **Build bootloader** - Start with Stage 1 boot.asm
4. **Add kernel core** - Get to kernel_main() in protected/long mode
5. **Add interrupts** - Handle exceptions
6. **Add scheduler** - Multiple processes with context switching
7. **Add drivers** - Serial, timer, disk
8. **Integrate EKOS** - Run EKOS as the primary application

---

**END OF LEARNING GUIDE**
