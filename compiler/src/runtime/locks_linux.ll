; A lock's word on Linux is a futex: 0 free, 1 taken, 2 taken with someone waiting, who sleeps in the kernel until
; it's given back (D348). The system call's number differs by processor, so it's filled in as FUTEX. 128 and 129 are
; FUTEX_WAIT_PRIVATE and FUTEX_WAKE_PRIVATE.

declare i64 @syscall(i64, ...)

define void @__cyborg_lock(ptr %word) {
entry:
  %pair = cmpxchg ptr %word, i32 0, i32 1 acquire monotonic
  %free = extractvalue { i32, i1 } %pair, 1
  br i1 %free, label %done, label %contended
contended:
  %old = atomicrmw xchg ptr %word, i32 2 acquire
  %taken = icmp eq i32 %old, 0
  br i1 %taken, label %done, label %wait
wait:
  %waited = call i64 (i64, ...) @syscall(i64 FUTEX, ptr %word, i32 128, i32 2, ptr null)
  br label %contended
done:
  ret void
}

define void @__cyborg_unlock(ptr %word) {
entry:
  %old = atomicrmw xchg ptr %word, i32 0 release
  %waiting = icmp eq i32 %old, 2
  br i1 %waiting, label %wake, label %done
wake:
  %woken = call i64 (i64, ...) @syscall(i64 FUTEX, ptr %word, i32 129, i32 1)
  br label %done
done:
  ret void
}
