
; Whether Linux says the processor has a feature: a bit of `getauxval(16)`, `AT_HWCAP`, or of `getauxval(26)`,
; `AT_HWCAP2` (D365).
declare i64 @getauxval(i64)

define i1 @__cyborg_has_hwcap(i64 %which, i64 %bit) {
entry:
  %caps = call i64 @getauxval(i64 %which)
  %shifted = lshr i64 %caps, %bit
  %low = and i64 %shifted, 1
  %has = icmp ne i64 %low, 0
  ret i1 %has
}
