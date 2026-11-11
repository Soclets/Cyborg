; Startup code for the Arduino Uno R4 WiFi: a Renesas RA4M1, a Cortex-M4 with hardware floats (D253, D261, D281).
; It's Cyborg's own, as D261 says each target's is. Not yet run on the real board, so treat it as untested.
;
; At reset the chip reads the stack's top and the reset handler from the vector table. The handler copies `.data`
; from flash to RAM, zeroes `.bss`, turns on the floating-point unit, which code built for hard floats needs, and
; runs the program. When `main` returns there's nothing to return to, so it waits forever.

@__cyborg_stack_top = external global i8
@__cyborg_data_start = external global i32
@__cyborg_data_end = external global i32
@__cyborg_data_load = external global i32
@__cyborg_bss_start = external global i32
@__cyborg_bss_end = external global i32

; The stack's top, then the 15 system exceptions. Every exception but reset waits forever, so a fault stops the board
; rather than running on.
@__cyborg_vectors = constant [16 x ptr] [
  ptr @__cyborg_stack_top, ptr @__cyborg_reset, ptr @__cyborg_halt, ptr @__cyborg_halt,
  ptr @__cyborg_halt, ptr @__cyborg_halt, ptr @__cyborg_halt, ptr null,
  ptr null, ptr null, ptr null, ptr @__cyborg_halt,
  ptr @__cyborg_halt, ptr null, ptr @__cyborg_halt, ptr @__cyborg_halt
], section ".isr_vector", align 4

@llvm.used = appending global [1 x ptr] [ptr @__cyborg_vectors], section "llvm.metadata"

declare i32 @main()

; The words are copied with volatile loads and stores, so LLVM can't turn the loops into calls to `memcpy` and
; `memset`, which a bare-metal program doesn't have.
define void @__cyborg_reset() noreturn {
entry:
  br label %copy
copy:
  %to = phi ptr [ @__cyborg_data_start, %entry ], [ %next_to, %copy_word ]
  %from = phi ptr [ @__cyborg_data_load, %entry ], [ %next_from, %copy_word ]
  %copied = icmp uge ptr %to, @__cyborg_data_end
  br i1 %copied, label %zero, label %copy_word
copy_word:
  %word = load volatile i32, ptr %from
  store volatile i32 %word, ptr %to
  %next_to = getelementptr i32, ptr %to, i32 1
  %next_from = getelementptr i32, ptr %from, i32 1
  br label %copy
zero:
  %at = phi ptr [ @__cyborg_bss_start, %copy ], [ %next_at, %zero_word ]
  %zeroed = icmp uge ptr %at, @__cyborg_bss_end
  br i1 %zeroed, label %start, label %zero_word
zero_word:
  store volatile i32 0, ptr %at
  %next_at = getelementptr i32, ptr %at, i32 1
  br label %zero
start:
  ; CPACR, at 0xE000ED88: full access to coprocessors 10 and 11, the floating-point unit.
  %cpacr = inttoptr i32 3758157192 to ptr
  %access = load volatile i32, ptr %cpacr
  %with_floats = or i32 %access, 15728640
  store volatile i32 %with_floats, ptr %cpacr
  fence seq_cst
  %ignored = call i32 @main()
  br label %wait
wait:
  br label %wait
}

define void @__cyborg_halt() noreturn {
entry:
  br label %wait
wait:
  br label %wait
}
