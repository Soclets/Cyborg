; What the runtime does the macOS way: how a lock waits (D348), and how a tuned program asks for its instructions (D365).
;
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

; Whether macOS says the processor has a feature, from `sysctlbyname` on its `hw.optional.arm.` name, which is 1
; when it's there and missing when the system doesn't know it (D365).
declare i32 @sysctlbyname(ptr, ptr, ptr, ptr, i64)

define i1 @__cyborg_has_sysctl(ptr %name) {
entry:
  %value = alloca i32
  %size = alloca i64
  store i32 0, ptr %value
  store i64 4, ptr %size
  %result = call i32 @sysctlbyname(ptr %name, ptr %value, ptr %size, ptr null, i64 0)
  %asked = icmp eq i32 %result, 0
  %got = load i32, ptr %value
  %set = icmp ne i32 %got, 0
  %has = and i1 %asked, %set
  ret i1 %has
}
