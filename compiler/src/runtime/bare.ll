; The runtime for a program with no operating system. What a bare-metal program prints to isn't decided (standard
; 12.2), so a failed check stops the processor with a trap rather than printing.

define void @__cyborg_trap() noreturn {
entry:
  call void @llvm.trap()
  unreachable
}

declare void @llvm.trap()
