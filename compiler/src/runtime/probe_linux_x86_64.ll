
; Whether the processor has a feature, from a bit of one register `cpuid` gives for a leaf, as 0 to 3 for `eax` to
; `edx`, once the processor says it has that leaf at all (D365).
define i1 @__cyborg_has_cpuid(i32 %leaf, i32 %register, i32 %bit) {
entry:
  %range = and i32 %leaf, 2147483648
  %top_answer = call { i32, i32, i32, i32 } asm "cpuid", "={ax},={bx},={cx},={dx},{ax},{cx}"(i32 %range, i32 0)
  %top = extractvalue { i32, i32, i32, i32 } %top_answer, 0
  %known = icmp ule i32 %leaf, %top
  br i1 %known, label %ask, label %missing
ask:
  %answer = call { i32, i32, i32, i32 } asm "cpuid", "={ax},={bx},={cx},={dx},{ax},{cx}"(i32 %leaf, i32 0)
  %a = extractvalue { i32, i32, i32, i32 } %answer, 0
  %b = extractvalue { i32, i32, i32, i32 } %answer, 1
  %c = extractvalue { i32, i32, i32, i32 } %answer, 2
  %d = extractvalue { i32, i32, i32, i32 } %answer, 3
  %is_b = icmp eq i32 %register, 1
  %is_c = icmp eq i32 %register, 2
  %is_d = icmp eq i32 %register, 3
  %ab = select i1 %is_b, i32 %b, i32 %a
  %abc = select i1 %is_c, i32 %c, i32 %ab
  %chosen = select i1 %is_d, i32 %d, i32 %abc
  %shifted = lshr i32 %chosen, %bit
  %low = and i32 %shifted, 1
  %has = icmp ne i32 %low, 0
  ret i1 %has
missing:
  ret i1 false
}
