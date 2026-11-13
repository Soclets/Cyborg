; A lock's word on macOS is an `os_unfair_lock`, which sleeps in the kernel while it waits (D348).

declare void @os_unfair_lock_lock(ptr)
declare void @os_unfair_lock_unlock(ptr)

define void @__cyborg_lock(ptr %word) {
entry:
  call void @os_unfair_lock_lock(ptr %word)
  ret void
}

define void @__cyborg_unlock(ptr %word) {
entry:
  call void @os_unfair_lock_unlock(ptr %word)
  ret void
}
