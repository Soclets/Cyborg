; Printing floats (D161, D162, D194), shared by every target. A float prints as the shortest text that reads back as
; the same value, and `.stored` prints every digit of what's in memory. Both come from the exact decimal digits of
; the value, found with big-integer arithmetic, so the result is the same bits on every machine (D158).
;
; A number's digits are a run of ASCII digits d1 d2 … dn, with no leading or trailing zeros, and a point P, so the
; number is 0.d1d2…dn × 10^P. Functions give back the count and the point packed in one i64: count << 32 | point.

declare void @llvm.memset.p0.i64(ptr, i8, i64, i1)
declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)
declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1)

define internal i64 @__cyborg_pack(i32 %count, i32 %point) {
entry:
  %count_wide = zext i32 %count to i64
  %high = shl i64 %count_wide, 32
  %point_wide = zext i32 %point to i64
  %packed = or i64 %high, %point_wide
  ret i64 %packed
}

; Writes an unsigned 64-bit integer's decimal digits at %out, giving how many.
define internal i32 @__cyborg_u64_digits(i64 %value, ptr %out) {
entry:
  %scratch = alloca [24 x i8]
  %end = getelementptr i8, ptr %scratch, i64 24
  br label %digit
digit:
  %rest = phi i64 [ %value, %entry ], [ %next, %digit ]
  %at = phi ptr [ %end, %entry ], [ %here, %digit ]
  %next = udiv i64 %rest, 10
  %remainder = urem i64 %rest, 10
  %code = add i64 %remainder, 48
  %byte = trunc i64 %code to i8
  %here = getelementptr i8, ptr %at, i64 -1
  store i8 %byte, ptr %here
  %more = icmp ne i64 %next, 0
  br i1 %more, label %digit, label %copy
copy:
  %start_int = ptrtoint ptr %here to i64
  %end_int = ptrtoint ptr %end to i64
  %count = sub i64 %end_int, %start_int
  call void @llvm.memcpy.p0.p0.i64(ptr %out, ptr %here, i64 %count, i1 false)
  %count32 = trunc i64 %count to i32
  ret i32 %count32
}

; The exact decimal digits of %m × 2^%e, for %m above 0 and below 2^60, written at %out, which holds 1200 bytes.
define internal i64 @__cyborg_exact_digits(i64 %m, i32 %e, ptr %out) {
entry:
  %limbs = alloca [40 x i32]
  call void @llvm.memset.p0.i64(ptr %limbs, i8 0, i64 160, i1 false)
  %raw = alloca [1240 x i8]
  %whole = icmp sge i32 %e, 0
  br i1 %whole, label %integer, label %fraction

; m × 2^e with e at least 0: a big integer, turned into digits 9 at a time by dividing by 10^9.
integer:
  %word = lshr i32 %e, 5
  %bit = and i32 %e, 31
  %bit64 = zext i32 %bit to i64
  %low_part = shl i64 %m, %bit64
  %high_shift = sub i64 64, %bit64
  %is_zero_bit = icmp eq i64 %bit64, 0
  %high_raw = lshr i64 %m, %high_shift
  %high_part = select i1 %is_zero_bit, i64 0, i64 %high_raw
  %word64 = zext i32 %word to i64
  %limb0_at = getelementptr i32, ptr %limbs, i64 %word64
  %limb0 = trunc i64 %low_part to i32
  store i32 %limb0, ptr %limb0_at
  %low_upper = lshr i64 %low_part, 32
  %limb1 = trunc i64 %low_upper to i32
  %limb1_at = getelementptr i32, ptr %limb0_at, i64 1
  store i32 %limb1, ptr %limb1_at
  %limb2 = trunc i64 %high_part to i32
  %limb2_at = getelementptr i32, ptr %limb0_at, i64 2
  store i32 %limb2, ptr %limb2_at
  %top_limb = add i32 %word, 2
  %chunks = alloca [140 x i32]
  br label %divide_round
divide_round:
  %chunk_count = phi i32 [ 0, %integer ], [ %chunk_next, %top_found ]
  %top = phi i32 [ %top_limb, %integer ], [ %top_after, %top_found ]
  br label %divide_limb
divide_limb:
  %index = phi i32 [ %top, %divide_round ], [ %index_next, %divide_limb ]
  %carry = phi i64 [ 0, %divide_round ], [ %carry_next, %divide_limb ]
  %index64 = zext i32 %index to i64
  %limb_at = getelementptr i32, ptr %limbs, i64 %index64
  %limb = load i32, ptr %limb_at
  %limb64 = zext i32 %limb to i64
  %carry_high = shl i64 %carry, 32
  %dividend = or i64 %carry_high, %limb64
  %quotient = udiv i64 %dividend, 1000000000
  %carry_next = urem i64 %dividend, 1000000000
  %quotient32 = trunc i64 %quotient to i32
  store i32 %quotient32, ptr %limb_at
  %index_next = sub i32 %index, 1
  %more_limbs = icmp sgt i32 %index, 0
  br i1 %more_limbs, label %divide_limb, label %divide_done
divide_done:
  %chunk_count64 = zext i32 %chunk_count to i64
  %chunk_at = getelementptr i32, ptr %chunks, i64 %chunk_count64
  %chunk = trunc i64 %carry_next to i32
  store i32 %chunk, ptr %chunk_at
  %chunk_next = add i32 %chunk_count, 1
  ; The top limb shrinks as the number does, and the loop ends when every limb is 0.
  br label %find_top
find_top:
  %probe = phi i32 [ %top, %divide_done ], [ %probe_next, %find_top_lower ]
  %probe64 = zext i32 %probe to i64
  %probe_at = getelementptr i32, ptr %limbs, i64 %probe64
  %probe_limb = load i32, ptr %probe_at
  %probe_nonzero = icmp ne i32 %probe_limb, 0
  br i1 %probe_nonzero, label %top_found, label %find_top_lower
find_top_lower:
  %probe_next = sub i32 %probe, 1
  %probe_more = icmp sgt i32 %probe, 0
  br i1 %probe_more, label %find_top, label %chunks_done
top_found:
  %top_after = add i32 %probe, 0
  br label %divide_round
chunks_done:
  ; Chunks are least significant first, so the last one is written first, without leading zeros.
  %last_chunk = sub i32 %chunk_next, 1
  %last64 = zext i32 %last_chunk to i64
  %first_chunk_at = getelementptr i32, ptr %chunks, i64 %last64
  %first_chunk = load i32, ptr %first_chunk_at
  %first_chunk64 = zext i32 %first_chunk to i64
  %first_count = call i32 @__cyborg_u64_digits(i64 %first_chunk64, ptr %raw)
  br label %chunk_write
chunk_write:
  %write_index = phi i32 [ %last_chunk, %chunks_done ], [ %write_next, %chunk_digit_done ]
  %written = phi i32 [ %first_count, %chunks_done ], [ %written_after, %chunk_digit_done ]
  %write_next = sub i32 %write_index, 1
  %has_chunk = icmp sge i32 %write_next, 0
  br i1 %has_chunk, label %chunk_digits, label %integer_done
chunk_digits:
  %write_next64 = zext i32 %write_next to i64
  %this_chunk_at = getelementptr i32, ptr %chunks, i64 %write_next64
  %this_chunk = load i32, ptr %this_chunk_at
  br label %chunk_digit
chunk_digit:
  %position = phi i32 [ 8, %chunk_digits ], [ %position_next, %chunk_digit ]
  %rest_chunk = phi i32 [ %this_chunk, %chunk_digits ], [ %rest_next, %chunk_digit ]
  %rest_next = udiv i32 %rest_chunk, 10
  %digit_value = urem i32 %rest_chunk, 10
  %digit_code = add i32 %digit_value, 48
  %digit_byte = trunc i32 %digit_code to i8
  %digit_place = add i32 %written, %position
  %digit_place64 = zext i32 %digit_place to i64
  %digit_at = getelementptr i8, ptr %raw, i64 %digit_place64
  store i8 %digit_byte, ptr %digit_at
  %position_next = sub i32 %position, 1
  %position_more = icmp sge i32 %position_next, 0
  br i1 %position_more, label %chunk_digit, label %chunk_digit_done
chunk_digit_done:
  %written_after = add i32 %written, 9
  br label %chunk_write
integer_done:
  %integer_packed = call i64 @__cyborg_trim(ptr %raw, i32 %written, i32 %written, ptr %out)
  ret i64 %integer_packed

; m × 2^e with e below 0: the whole part, then the fraction, shifted to fill whole limbs, multiplied by 10 again and
; again, each carry out of the top being the next digit.
fraction:
  %shift = sub i32 0, %e
  %shift64 = zext i32 %shift to i64
  %small_shift = icmp ult i32 %shift, 64
  %whole_raw = lshr i64 %m, %shift64
  %whole_part = select i1 %small_shift, i64 %whole_raw, i64 0
  %mask_raw = shl i64 1, %shift64
  %mask_minus = sub i64 %mask_raw, 1
  %mask = select i1 %small_shift, i64 %mask_minus, i64 -1
  %fraction_part = and i64 %m, %mask
  %limb_total = add i32 %shift, 31
  %limb_count = lshr i32 %limb_total, 5
  %filled = shl i32 %limb_count, 5
  %pad = sub i32 %filled, %shift
  %pad64 = zext i32 %pad to i64
  %placed_low = shl i64 %fraction_part, %pad64
  %pad_zero = icmp eq i64 %pad64, 0
  %placed_high_shift = sub i64 64, %pad64
  %placed_high_raw = lshr i64 %fraction_part, %placed_high_shift
  %placed_high = select i1 %pad_zero, i64 0, i64 %placed_high_raw
  %f_limb0 = trunc i64 %placed_low to i32
  store i32 %f_limb0, ptr %limbs
  %placed_low_upper = lshr i64 %placed_low, 32
  %f_limb1 = trunc i64 %placed_low_upper to i32
  %f_limb1_at = getelementptr i32, ptr %limbs, i64 1
  store i32 %f_limb1, ptr %f_limb1_at
  %f_limb2 = trunc i64 %placed_high to i32
  %f_limb2_at = getelementptr i32, ptr %limbs, i64 2
  store i32 %f_limb2, ptr %f_limb2_at
  %whole_nonzero = icmp ne i64 %whole_part, 0
  br i1 %whole_nonzero, label %whole_digits, label %fraction_start
whole_digits:
  %whole_count = call i32 @__cyborg_u64_digits(i64 %whole_part, ptr %raw)
  br label %fraction_start
fraction_start:
  %start_count = phi i32 [ 0, %fraction ], [ %whole_count, %whole_digits ]
  br label %fraction_digit
fraction_digit:
  %done_digits = phi i32 [ 0, %fraction_start ], [ %done_next, %multiply_done ]
  br label %multiply_limb
multiply_limb:
  %m_index = phi i32 [ 0, %fraction_digit ], [ %m_index_next, %multiply_limb ]
  %m_carry = phi i64 [ 0, %fraction_digit ], [ %m_carry_next, %multiply_limb ]
  %m_index64 = zext i32 %m_index to i64
  %m_limb_at = getelementptr i32, ptr %limbs, i64 %m_index64
  %m_limb = load i32, ptr %m_limb_at
  %m_limb64 = zext i32 %m_limb to i64
  %m_product = mul i64 %m_limb64, 10
  %m_sum = add i64 %m_product, %m_carry
  %m_low = trunc i64 %m_sum to i32
  store i32 %m_low, ptr %m_limb_at
  %m_carry_next = lshr i64 %m_sum, 32
  %m_index_next = add i32 %m_index, 1
  %m_more = icmp ult i32 %m_index_next, %limb_count
  br i1 %m_more, label %multiply_limb, label %multiply_done
multiply_done:
  %fraction_code = add i64 %m_carry_next, 48
  %fraction_byte = trunc i64 %fraction_code to i8
  %fraction_place = add i32 %start_count, %done_digits
  %fraction_place64 = zext i32 %fraction_place to i64
  %fraction_at = getelementptr i8, ptr %raw, i64 %fraction_place64
  store i8 %fraction_byte, ptr %fraction_at
  %done_next = add i32 %done_digits, 1
  %fraction_more = icmp ult i32 %done_next, %shift
  br i1 %fraction_more, label %fraction_digit, label %fraction_done
fraction_done:
  %total = add i32 %start_count, %shift
  %fraction_packed = call i64 @__cyborg_trim(ptr %raw, i32 %total, i32 %start_count, ptr %out)
  ret i64 %fraction_packed
}

; Copies digits to %out without leading or trailing zeros. %point is where the point sits among the %count digits
; as written, counting from the left.
define internal i64 @__cyborg_trim(ptr %raw, i32 %count, i32 %point, ptr %out) {
entry:
  br label %leading
leading:
  %first = phi i32 [ 0, %entry ], [ %first_next, %leading_zero ]
  %at_end = icmp sge i32 %first, %count
  br i1 %at_end, label %zero, label %leading_check
leading_check:
  %first64 = zext i32 %first to i64
  %first_at = getelementptr i8, ptr %raw, i64 %first64
  %first_digit = load i8, ptr %first_at
  %is_zero = icmp eq i8 %first_digit, 48
  br i1 %is_zero, label %leading_zero, label %trailing
leading_zero:
  %first_next = add i32 %first, 1
  br label %leading
trailing:
  %last = phi i32 [ %count, %leading_check ], [ %last_next, %trailing_zero ]
  %last_index = sub i32 %last, 1
  %last64 = zext i32 %last_index to i64
  %last_at = getelementptr i8, ptr %raw, i64 %last64
  %last_digit = load i8, ptr %last_at
  %last_is_zero = icmp eq i8 %last_digit, 48
  br i1 %last_is_zero, label %trailing_zero, label %copy
trailing_zero:
  %last_next = sub i32 %last, 1
  br label %trailing
copy:
  %kept = sub i32 %last, %first
  %kept64 = zext i32 %kept to i64
  call void @llvm.memmove.p0.p0.i64(ptr %out, ptr %first_at, i64 %kept64, i1 false)
  %new_point = sub i32 %point, %first
  %packed = call i64 @__cyborg_pack(i32 %kept, i32 %new_point)
  ret i64 %packed
zero:
  ret i64 0
}

; The digit of a number at place %place, where place 0 is its first digit and later digits count up.
define internal i32 @__cyborg_digit(ptr %digits, i32 %count, i32 %index) {
entry:
  %inside_low = icmp sge i32 %index, 0
  %inside_high = icmp slt i32 %index, %count
  %inside = and i1 %inside_low, %inside_high
  br i1 %inside, label %read, label %none
read:
  %index64 = sext i32 %index to i64
  %at = getelementptr i8, ptr %digits, i64 %index64
  %byte = load i8, ptr %at
  %value = sext i8 %byte to i32
  %digit = sub i32 %value, 48
  ret i32 %digit
none:
  ret i32 0
}

; Compares two positive numbers given by digits and points: -1, 0 or 1. A count of 0 is zero.
define internal i32 @__cyborg_compare(ptr %a, i32 %a_count, i32 %a_point, ptr %b, i32 %b_count, i32 %b_point) {
entry:
  %a_zero = icmp eq i32 %a_count, 0
  %b_zero = icmp eq i32 %b_count, 0
  %both_zero = and i1 %a_zero, %b_zero
  br i1 %both_zero, label %equal, label %not_both
not_both:
  br i1 %a_zero, label %less, label %a_nonzero
a_nonzero:
  br i1 %b_zero, label %greater, label %points
points:
  %a_higher = icmp sgt i32 %a_point, %b_point
  br i1 %a_higher, label %greater, label %points_lower
points_lower:
  %a_lower = icmp slt i32 %a_point, %b_point
  br i1 %a_lower, label %less, label %digits
digits:
  %longest = call i32 @llvm.smax.i32(i32 %a_count, i32 %b_count)
  br label %loop
loop:
  %index = phi i32 [ 0, %digits ], [ %next, %same ]
  %finished = icmp sge i32 %index, %longest
  br i1 %finished, label %equal, label %step
step:
  %a_digit = call i32 @__cyborg_digit(ptr %a, i32 %a_count, i32 %index)
  %b_digit = call i32 @__cyborg_digit(ptr %b, i32 %b_count, i32 %index)
  %a_more = icmp sgt i32 %a_digit, %b_digit
  br i1 %a_more, label %greater, label %step_less
step_less:
  %a_less = icmp slt i32 %a_digit, %b_digit
  br i1 %a_less, label %less, label %same
same:
  %next = add i32 %index, 1
  br label %loop
equal:
  ret i32 0
less:
  ret i32 -1
greater:
  ret i32 1
}

declare i32 @llvm.smax.i32(i32, i32)

; Keeps the first %keep digits of a number, adding 1 at the last one if %up says so, and writes the result to %out.
; Gives the packed count and point of the result, which can gain a digit, as 9.9 rounding up to 10.
define internal i64 @__cyborg_cut(ptr %digits, i32 %count, i32 %point, i32 %keep, i1 %up, ptr %out) {
entry:
  %nothing_kept = icmp sle i32 %keep, 0
  br i1 %nothing_kept, label %below_first, label %copy
below_first:
  ; Cutting before the first digit leaves 0, or 1 at the cut's place when rounding up.
  br i1 %up, label %one_at_cut, label %zero_result
one_at_cut:
  store i8 49, ptr %out
  %cut_point = sub i32 %point, %keep
  %cut_point_one = add i32 %cut_point, 1
  %one_packed = call i64 @__cyborg_pack(i32 1, i32 %cut_point_one)
  ret i64 %one_packed
zero_result:
  ret i64 0
copy:
  br label %copy_digit
copy_digit:
  %index = phi i32 [ 0, %copy ], [ %index_next, %copy_digit ]
  %digit = call i32 @__cyborg_digit(ptr %digits, i32 %count, i32 %index)
  %code = add i32 %digit, 48
  %byte = trunc i32 %code to i8
  %index64 = sext i32 %index to i64
  %out_at = getelementptr i8, ptr %out, i64 %index64
  store i8 %byte, ptr %out_at
  %index_next = add i32 %index, 1
  %copy_more = icmp slt i32 %index_next, %keep
  br i1 %copy_more, label %copy_digit, label %copied
copied:
  br i1 %up, label %carry_start, label %finish
carry_start:
  br label %carry
carry:
  %carry_index = phi i32 [ %keep, %carry_start ], [ %carry_index_next, %carry_nine ]
  %carry_index_next = sub i32 %carry_index, 1
  %carry_past_start = icmp slt i32 %carry_index_next, 0
  br i1 %carry_past_start, label %all_nines, label %carry_digit
carry_digit:
  %carry64 = sext i32 %carry_index_next to i64
  %carry_at = getelementptr i8, ptr %out, i64 %carry64
  %carry_byte = load i8, ptr %carry_at
  %is_nine = icmp eq i8 %carry_byte, 57
  br i1 %is_nine, label %carry_nine, label %carry_add
carry_nine:
  store i8 48, ptr %carry_at
  br label %carry
carry_add:
  %added = add i8 %carry_byte, 1
  store i8 %added, ptr %carry_at
  br label %finish
all_nines:
  ; Every digit was 9, so the result is 1 followed by zeros, one place higher.
  store i8 49, ptr %out
  %higher = add i32 %point, 1
  %nines_packed = call i64 @__cyborg_trim(ptr %out, i32 %keep, i32 %higher, ptr %out)
  ret i64 %nines_packed
finish:
  %finished = call i64 @__cyborg_trim(ptr %out, i32 %keep, i32 %point, ptr %out)
  ret i64 %finished
}

; How the digits after the first %keep compare with a half: -1 below, 0 exactly half, 1 above.
define internal i32 @__cyborg_rest_against_half(ptr %digits, i32 %count, i32 %keep) {
entry:
  %next = call i32 @__cyborg_digit(ptr %digits, i32 %count, i32 %keep)
  %above = icmp sgt i32 %next, 5
  br i1 %above, label %greater, label %not_above
not_above:
  %below = icmp slt i32 %next, 5
  br i1 %below, label %less, label %five
five:
  ; Exactly half only when nothing nonzero follows the 5.
  %after = add i32 %keep, 1
  %more = icmp sgt i32 %count, %after
  br i1 %more, label %greater, label %equal
equal:
  ret i32 0
less:
  ret i32 -1
greater:
  ret i32 1
}

; Whether a number lies in the rounding interval between low and high, which includes its ends when %inclusive.
define internal i1 @__cyborg_inside(ptr %value, i64 %packed, ptr %low, i64 %low_packed, ptr %high, i64 %high_packed, i1 %inclusive) {
entry:
  %count = call i32 @__cyborg_count(i64 %packed)
  %point = call i32 @__cyborg_point(i64 %packed)
  %low_count = call i32 @__cyborg_count(i64 %low_packed)
  %low_point = call i32 @__cyborg_point(i64 %low_packed)
  %high_count = call i32 @__cyborg_count(i64 %high_packed)
  %high_point = call i32 @__cyborg_point(i64 %high_packed)
  %against_low = call i32 @__cyborg_compare(ptr %value, i32 %count, i32 %point, ptr %low, i32 %low_count, i32 %low_point)
  %against_high = call i32 @__cyborg_compare(ptr %value, i32 %count, i32 %point, ptr %high, i32 %high_count, i32 %high_point)
  %above_low = icmp sgt i32 %against_low, 0
  %at_low = icmp eq i32 %against_low, 0
  %below_high = icmp slt i32 %against_high, 0
  %at_high = icmp eq i32 %against_high, 0
  %low_ok_inclusive = or i1 %above_low, %at_low
  %low_ok = select i1 %inclusive, i1 %low_ok_inclusive, i1 %above_low
  %high_ok_inclusive = or i1 %below_high, %at_high
  %high_ok = select i1 %inclusive, i1 %high_ok_inclusive, i1 %below_high
  %inside = and i1 %low_ok, %high_ok
  ret i1 %inside
}

define internal i32 @__cyborg_count(i64 %packed) {
entry:
  %high = lshr i64 %packed, 32
  %count = trunc i64 %high to i32
  ret i32 %count
}

define internal i32 @__cyborg_point(i64 %packed) {
entry:
  %point = trunc i64 %packed to i32
  ret i32 %point
}

; The shortest digits that read back as m × 2^e, for a float whose neighbours are halfway to the next value down
; and up. %narrow_below says the gap below is half the gap above, as at a power of two. A shorter answer is always
; one of the two numbers of that length either side of the value, so each length checks just those two (D161).
define internal i64 @__cyborg_shortest(i64 %m, i32 %e, i1 %narrow_below, ptr %out) {
entry:
  %exact = alloca [1240 x i8]
  %low = alloca [1240 x i8]
  %high = alloca [1240 x i8]
  %down = alloca [32 x i8]
  %up = alloca [32 x i8]
  %exact_packed = call i64 @__cyborg_exact_digits(i64 %m, i32 %e, ptr %exact)
  %twice = shl i64 %m, 1
  %below_e = sub i32 %e, 1
  %high_m = add i64 %twice, 1
  %high_packed = call i64 @__cyborg_exact_digits(i64 %high_m, i32 %below_e, ptr %high)
  br i1 %narrow_below, label %narrow, label %even_gap
narrow:
  %four = shl i64 %m, 2
  %narrow_m = sub i64 %four, 1
  %two_below = sub i32 %e, 2
  %narrow_packed = call i64 @__cyborg_exact_digits(i64 %narrow_m, i32 %two_below, ptr %low)
  br label %bounds_ready
even_gap:
  %even_m = sub i64 %twice, 1
  %even_packed = call i64 @__cyborg_exact_digits(i64 %even_m, i32 %below_e, ptr %low)
  br label %bounds_ready
bounds_ready:
  %low_packed = phi i64 [ %narrow_packed, %narrow ], [ %even_packed, %even_gap ]
  %odd = and i64 %m, 1
  %inclusive = icmp eq i64 %odd, 0
  %exact_count = call i32 @__cyborg_count(i64 %exact_packed)
  %exact_point = call i32 @__cyborg_point(i64 %exact_packed)
  br label %length
length:
  %keep = phi i32 [ 1, %bounds_ready ], [ %keep_next, %next_length ]
  %all_digits = icmp sge i32 %keep, %exact_count
  br i1 %all_digits, label %use_exact, label %try
try:
  %down_packed = call i64 @__cyborg_cut(ptr %exact, i32 %exact_count, i32 %exact_point, i32 %keep, i1 false, ptr %down)
  %up_packed = call i64 @__cyborg_cut(ptr %exact, i32 %exact_count, i32 %exact_point, i32 %keep, i1 true, ptr %up)
  %down_in = call i1 @__cyborg_inside(ptr %down, i64 %down_packed, ptr %low, i64 %low_packed, ptr %high, i64 %high_packed, i1 %inclusive)
  %up_in = call i1 @__cyborg_inside(ptr %up, i64 %up_packed, ptr %low, i64 %low_packed, ptr %high, i64 %high_packed, i1 %inclusive)
  %both = and i1 %down_in, %up_in
  br i1 %both, label %pick_closer, label %one_or_none
pick_closer:
  %rest = call i32 @__cyborg_rest_against_half(ptr %exact, i32 %exact_count, i32 %keep)
  %closer_up = icmp sgt i32 %rest, 0
  br i1 %closer_up, label %take_up, label %take_down
one_or_none:
  br i1 %down_in, label %take_down, label %up_alone
up_alone:
  br i1 %up_in, label %take_up, label %next_length
next_length:
  %keep_next = add i32 %keep, 1
  br label %length
take_down:
  %down_count = call i32 @__cyborg_count(i64 %down_packed)
  %down_count64 = zext i32 %down_count to i64
  call void @llvm.memcpy.p0.p0.i64(ptr %out, ptr %down, i64 %down_count64, i1 false)
  ret i64 %down_packed
take_up:
  %up_count = call i32 @__cyborg_count(i64 %up_packed)
  %up_count64 = zext i32 %up_count to i64
  call void @llvm.memcpy.p0.p0.i64(ptr %out, ptr %up, i64 %up_count64, i1 false)
  ret i64 %up_packed
use_exact:
  %exact_count64 = zext i32 %exact_count to i64
  call void @llvm.memcpy.p0.p0.i64(ptr %out, ptr %exact, i64 %exact_count64, i1 false)
  ret i64 %exact_packed
}

; Writes a byte at %out[%at] and gives the next place.
define internal i32 @__cyborg_put(ptr %out, i32 %at, i8 %byte) {
entry:
  %at64 = sext i32 %at to i64
  %place = getelementptr i8, ptr %out, i64 %at64
  store i8 %byte, ptr %place
  %next = add i32 %at, 1
  ret i32 %next
}

; Writes digits in plain notation: `0.0012`, `12.5`, `1500.0`. With %places at 0 or above, exactly that many digits
; follow the point, and none means no point; below 0, at least one does.
define internal i32 @__cyborg_plain(ptr %digits, i32 %count, i32 %point, i32 %places, ptr %out, i32 %start) {
entry:
  %whole_digits = icmp sgt i32 %point, 0
  br i1 %whole_digits, label %whole, label %zero_whole
zero_whole:
  %after_zero = call i32 @__cyborg_put(ptr %out, i32 %start, i8 48)
  br label %fraction
whole:
  br label %whole_digit
whole_digit:
  %whole_index = phi i32 [ 0, %whole ], [ %whole_next, %whole_digit ]
  %whole_at = phi i32 [ %start, %whole ], [ %whole_written, %whole_digit ]
  %whole_value = call i32 @__cyborg_digit(ptr %digits, i32 %count, i32 %whole_index)
  %whole_code = add i32 %whole_value, 48
  %whole_byte = trunc i32 %whole_code to i8
  %whole_written = call i32 @__cyborg_put(ptr %out, i32 %whole_at, i8 %whole_byte)
  %whole_next = add i32 %whole_index, 1
  %whole_more = icmp slt i32 %whole_next, %point
  br i1 %whole_more, label %whole_digit, label %fraction
fraction:
  %at_fraction = phi i32 [ %after_zero, %zero_whole ], [ %whole_written, %whole_digit ]
  ; Digits after the point: as many as the number has, at least one, or exactly %places.
  %after_point = sub i32 %count, %point
  %some_after = call i32 @llvm.smax.i32(i32 %after_point, i32 1)
  %fixed = icmp sge i32 %places, 0
  %wanted = select i1 %fixed, i32 %places, i32 %some_after
  %none_wanted = icmp eq i32 %wanted, 0
  br i1 %none_wanted, label %done, label %point_mark
point_mark:
  %after_mark = call i32 @__cyborg_put(ptr %out, i32 %at_fraction, i8 46)
  br label %fraction_digit
fraction_digit:
  %fraction_index = phi i32 [ 0, %point_mark ], [ %fraction_next, %fraction_digit ]
  %fraction_at = phi i32 [ %after_mark, %point_mark ], [ %fraction_written, %fraction_digit ]
  %digit_index = add i32 %point, %fraction_index
  %fraction_value = call i32 @__cyborg_digit(ptr %digits, i32 %count, i32 %digit_index)
  %fraction_code = add i32 %fraction_value, 48
  %fraction_byte = trunc i32 %fraction_code to i8
  %fraction_written = call i32 @__cyborg_put(ptr %out, i32 %fraction_at, i8 %fraction_byte)
  %fraction_next = add i32 %fraction_index, 1
  %fraction_more = icmp slt i32 %fraction_next, %wanted
  br i1 %fraction_more, label %fraction_digit, label %done
done:
  %end = phi i32 [ %at_fraction, %fraction ], [ %fraction_written, %fraction_digit ]
  ret i32 %end
}

; Formats a float into %out, giving the text's length, or -1 when `places:` falls exactly halfway, or -2 when
; `form: science` would need an exponent of 0 or below, since neither is decided (standard 12.2).
; %bits is 32 or 64. %form is 0 for the shortest text, 1 for every stored digit, 2 for `places:`, 3 for
; `form: science` and 4 for both. %sign writes `+` in front of a value that isn't negative.
define i64 @__cyborg_format_float(double %value, i32 %bits, i32 %form, i32 %places, i1 %sign, ptr %out) {
entry:
  %digits = alloca [1240 x i8]
  %rounded = alloca [1240 x i8]
  %is_nan = fcmp uno double %value, %value
  br i1 %is_nan, label %nan, label %not_nan
nan:
  store i8 78, ptr %out
  %nan_a = getelementptr i8, ptr %out, i64 1
  store i8 97, ptr %nan_a
  %nan_n = getelementptr i8, ptr %out, i64 2
  store i8 78, ptr %nan_n
  ret i64 3
not_nan:
  %raw = bitcast double %value to i64
  %negative = icmp slt i64 %raw, 0
  %magnitude = call double @llvm.fabs.f64(double %value)
  %is_infinite = fcmp oeq double %magnitude, 0x7FF0000000000000
  %sign_byte = select i1 %negative, i8 45, i8 43
  %shows_sign = or i1 %negative, %sign
  %first = select i1 %shows_sign, i32 1, i32 0
  br i1 %shows_sign, label %write_sign, label %signed
write_sign:
  store i8 %sign_byte, ptr %out
  br label %signed
signed:
  br i1 %is_infinite, label %infinite, label %finite
infinite:
  %inf_i = call i32 @__cyborg_put(ptr %out, i32 %first, i8 105)
  %inf_n = call i32 @__cyborg_put(ptr %out, i32 %inf_i, i8 110)
  %inf_f = call i32 @__cyborg_put(ptr %out, i32 %inf_n, i8 102)
  %inf_length = sext i32 %inf_f to i64
  ret i64 %inf_length
finite:
  ; Split the value into m × 2^e, at the width it's stored in.
  %is_32 = icmp eq i32 %bits, 32
  br i1 %is_32, label %split_32, label %split_64
split_64:
  %raw_magnitude = and i64 %raw, 9223372036854775807
  %exponent_field = lshr i64 %raw_magnitude, 52
  %fraction_field = and i64 %raw_magnitude, 4503599627370495
  %subnormal = icmp eq i64 %exponent_field, 0
  %with_hidden = or i64 %fraction_field, 4503599627370496
  %m64 = select i1 %subnormal, i64 %fraction_field, i64 %with_hidden
  %exponent_field32 = trunc i64 %exponent_field to i32
  %e_normal = sub i32 %exponent_field32, 1075
  %e64 = select i1 %subnormal, i32 -1074, i32 %e_normal
  %power_of_two = icmp eq i64 %fraction_field, 0
  %above_lowest = icmp ugt i64 %exponent_field, 1
  %narrow64 = and i1 %power_of_two, %above_lowest
  br label %split
split_32:
  %single = fptrunc double %magnitude to float
  %raw32 = bitcast float %single to i32
  %exponent_field_32 = lshr i32 %raw32, 23
  %fraction_field_32 = and i32 %raw32, 8388607
  %subnormal_32 = icmp eq i32 %exponent_field_32, 0
  %with_hidden_32 = or i32 %fraction_field_32, 8388608
  %m32_narrow = select i1 %subnormal_32, i32 %fraction_field_32, i32 %with_hidden_32
  %m32 = zext i32 %m32_narrow to i64
  %e_normal_32 = sub i32 %exponent_field_32, 150
  %e32 = select i1 %subnormal_32, i32 -149, i32 %e_normal_32
  %power_of_two_32 = icmp eq i32 %fraction_field_32, 0
  %above_lowest_32 = icmp ugt i32 %exponent_field_32, 1
  %narrow32 = and i1 %power_of_two_32, %above_lowest_32
  br label %split
split:
  %m = phi i64 [ %m64, %split_64 ], [ %m32, %split_32 ]
  %e = phi i32 [ %e64, %split_64 ], [ %e32, %split_32 ]
  %narrow = phi i1 [ %narrow64, %split_64 ], [ %narrow32, %split_32 ]
  %zero = icmp eq i64 %m, 0
  br i1 %zero, label %zero_value, label %nonzero
zero_value:
  %zero_places = icmp sge i32 %places, 0
  %fixed_zero = icmp eq i32 %form, 2
  %plain_zero_places = select i1 %fixed_zero, i32 %places, i32 -1
  %zero_end = call i32 @__cyborg_plain(ptr %digits, i32 0, i32 1, i32 %plain_zero_places, ptr %out, i32 %first)
  %zero_length = sext i32 %zero_end to i64
  %science_zero = icmp sge i32 %form, 3
  %zero_result = select i1 %science_zero, i64 -2, i64 %zero_length
  ret i64 %zero_result
nonzero:
  %wants_exact = icmp eq i32 %form, 1
  %wants_rounding = icmp eq i32 %form, 2
  %wants_science_places = icmp eq i32 %form, 4
  %needs_exact = or i1 %wants_exact, %wants_rounding
  %needs_exact_all = or i1 %needs_exact, %wants_science_places
  br i1 %needs_exact_all, label %exact, label %shortest
exact:
  %exact_packed = call i64 @__cyborg_exact_digits(i64 %m, i32 %e, ptr %digits)
  br label %have_digits
shortest:
  %shortest_packed = call i64 @__cyborg_shortest(i64 %m, i32 %e, i1 %narrow, ptr %digits)
  br label %have_digits
have_digits:
  %packed = phi i64 [ %exact_packed, %exact ], [ %shortest_packed, %shortest ]
  %count = call i32 @__cyborg_count(i64 %packed)
  %point = call i32 @__cyborg_point(i64 %packed)
  switch i32 %form, label %plain_form [ i32 2, label %round_places
                                        i32 3, label %science
                                        i32 4, label %round_science ]
plain_form:
  %plain_end = call i32 @__cyborg_plain(ptr %digits, i32 %count, i32 %point, i32 -1, ptr %out, i32 %first)
  %plain_length = sext i32 %plain_end to i64
  ret i64 %plain_length
round_places:
  ; Keep the digits down to %places after the point, rounding the rest, and refuse exactly half.
  %keep_places = add i32 %point, %places
  %against_half = call i32 @__cyborg_rest_against_half(ptr %digits, i32 %count, i32 %keep_places)
  %no_rest = icmp sge i32 %keep_places, %count
  %tie = icmp eq i32 %against_half, 0
  %real_tie = and i1 %tie, true
  %tie_and_rest = select i1 %no_rest, i1 false, i1 %real_tie
  br i1 %tie_and_rest, label %halfway, label %rounding
rounding:
  %round_up_places = icmp sgt i32 %against_half, 0
  %round_up = select i1 %no_rest, i1 false, i1 %round_up_places
  %places_packed = call i64 @__cyborg_cut(ptr %digits, i32 %count, i32 %point, i32 %keep_places, i1 %round_up, ptr %rounded)
  %places_count = call i32 @__cyborg_count(i64 %places_packed)
  %places_point_raw = call i32 @__cyborg_point(i64 %places_packed)
  %places_is_zero = icmp eq i32 %places_count, 0
  %places_point = select i1 %places_is_zero, i32 1, i32 %places_point_raw
  %places_end = call i32 @__cyborg_plain(ptr %rounded, i32 %places_count, i32 %places_point, i32 %places, ptr %out, i32 %first)
  %places_length = sext i32 %places_end to i64
  ret i64 %places_length
halfway:
  ret i64 -1
round_science:
  ; Keep %places + 1 significant digits.
  %keep_science = add i32 %places, 1
  %science_half = call i32 @__cyborg_rest_against_half(ptr %digits, i32 %count, i32 %keep_science)
  %science_no_rest = icmp sge i32 %keep_science, %count
  %science_tie_raw = icmp eq i32 %science_half, 0
  %science_tie = select i1 %science_no_rest, i1 false, i1 %science_tie_raw
  br i1 %science_tie, label %halfway, label %science_rounding
science_rounding:
  %science_up_raw = icmp sgt i32 %science_half, 0
  %science_up = select i1 %science_no_rest, i1 false, i1 %science_up_raw
  %science_packed = call i64 @__cyborg_cut(ptr %digits, i32 %count, i32 %point, i32 %keep_science, i1 %science_up, ptr %rounded)
  %science_count = call i32 @__cyborg_count(i64 %science_packed)
  %science_point = call i32 @__cyborg_point(i64 %science_packed)
  %science_end = call i64 @__cyborg_science(ptr %rounded, i32 %science_count, i32 %science_point, i32 %places, ptr %out, i32 %first)
  ret i64 %science_end
science:
  %science_plain = call i64 @__cyborg_science(ptr %digits, i32 %count, i32 %point, i32 -1, ptr %out, i32 %first)
  ret i64 %science_plain
}

; Writes `d.ddde{exponent}`, with exactly %places digits after the point when that's 0 or above, and at least one
; otherwise. An exponent of 0 or below isn't decided (standard 12.2), so that gives -2.
define internal i64 @__cyborg_science(ptr %digits, i32 %count, i32 %point, i32 %places, ptr %out, i32 %start) {
entry:
  %exponent = sub i32 %point, 1
  %positive = icmp sgt i32 %exponent, 0
  br i1 %positive, label %write, label %refused
refused:
  ret i64 -2
write:
  %first_digit = call i32 @__cyborg_digit(ptr %digits, i32 %count, i32 0)
  %first_code = add i32 %first_digit, 48
  %first_byte = trunc i32 %first_code to i8
  %after_first = call i32 @__cyborg_put(ptr %out, i32 %start, i8 %first_byte)
  %after_point = call i32 @__cyborg_put(ptr %out, i32 %after_first, i8 46)
  %rest = sub i32 %count, 1
  %some = call i32 @llvm.smax.i32(i32 %rest, i32 1)
  %fixed = icmp sge i32 %places, 0
  %wanted = select i1 %fixed, i32 %places, i32 %some
  br label %digit
digit:
  %index = phi i32 [ 0, %write ], [ %index_next, %digit ]
  %at = phi i32 [ %after_point, %write ], [ %written, %digit ]
  %which = add i32 %index, 1
  %value = call i32 @__cyborg_digit(ptr %digits, i32 %count, i32 %which)
  %code = add i32 %value, 48
  %byte = trunc i32 %code to i8
  %written = call i32 @__cyborg_put(ptr %out, i32 %at, i8 %byte)
  %index_next = add i32 %index, 1
  %more = icmp slt i32 %index_next, %wanted
  br i1 %more, label %digit, label %exponent_part
exponent_part:
  %after_e = call i32 @__cyborg_put(ptr %out, i32 %written, i8 101)
  %exponent64 = sext i32 %exponent to i64
  %after_e64 = sext i32 %after_e to i64
  %exponent_at = getelementptr i8, ptr %out, i64 %after_e64
  %exponent_count = call i32 @__cyborg_u64_digits(i64 %exponent64, ptr %exponent_at)
  %end = add i32 %after_e, %exponent_count
  %end64 = sext i32 %end to i64
  ret i64 %end64
}

declare double @llvm.fabs.f64(double)
