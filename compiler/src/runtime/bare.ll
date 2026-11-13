; The runtime for a program with no operating system. What a bare-metal program prints to isn't decided (standard
; 12.2), so a failed check stops the processor with a trap rather than printing.

define void @__cyborg_trap() noreturn {
entry:
  call void @llvm.trap()
  unreachable
}

declare void @llvm.trap()

; With one thread, nothing else can hold a lock, so taking one does nothing (D348).
define void @__cyborg_lock_all(ptr %words, i64 %count) {
entry:
  ret void
}

define void @__cyborg_unlock_all(ptr %words, i64 %count) {
entry:
  ret void
}
