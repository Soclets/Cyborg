; The runtime for a program on an operating system: writers (D272, D273), crashing (D96, D274) and printing (D194).
; It's LLVM IR, read through Lyftar (rule 1), and linked into each program before optimizing, so its small functions
; inline into the code that calls them. The C library's `write`, `malloc`, `free` and `exit` come from libSystem on
; macOS and from musl on Linux (D261).

; A writer: its file descriptor, its buffer, the buffer's size and how much it holds, and its neighbours in the list
; of buffered writers, which a crash flushes before its message (D274). An unbuffered writer has no buffer.
%writer = type { i32, ptr, i64, i64, ptr, ptr }

@__cyborg_writers = internal global ptr null

declare i64 @write(i32, ptr, i64)
declare ptr @malloc(i64)
declare void @free(ptr)
declare void @exit(i32) noreturn
declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)

; Writes every byte, carrying on after a short write.
define void @__cyborg_write_all(i32 %descriptor, ptr %data, i64 %length) {
entry:
  br label %loop
loop:
  %at = phi ptr [ %data, %entry ], [ %next_at, %wrote ]
  %left = phi i64 [ %length, %entry ], [ %next_left, %wrote ]
  %finished = icmp sle i64 %left, 0
  br i1 %finished, label %exit, label %write
write:
  %count = call i64 @write(i32 %descriptor, ptr %at, i64 %left)
  %failed = icmp slt i64 %count, 1
  br i1 %failed, label %exit, label %wrote
wrote:
  %next_at = getelementptr i8, ptr %at, i64 %count
  %next_left = sub i64 %left, %count
  br label %loop
exit:
  ret void
}

define void @__cyborg_writer_open(ptr %writer, i32 %descriptor, i64 %capacity) {
entry:
  %descriptor_at = getelementptr %writer, ptr %writer, i32 0, i32 0
  store i32 %descriptor, ptr %descriptor_at
  %capacity_at = getelementptr %writer, ptr %writer, i32 0, i32 2
  store i64 %capacity, ptr %capacity_at
  %length_at = getelementptr %writer, ptr %writer, i32 0, i32 3
  store i64 0, ptr %length_at
  %buffer_at = getelementptr %writer, ptr %writer, i32 0, i32 1
  %next_at = getelementptr %writer, ptr %writer, i32 0, i32 4
  %previous_at = getelementptr %writer, ptr %writer, i32 0, i32 5
  store ptr null, ptr %previous_at
  %buffered = icmp sgt i64 %capacity, 0
  br i1 %buffered, label %allocate, label %unbuffered
allocate:
  %buffer = call ptr @malloc(i64 %capacity)
  store ptr %buffer, ptr %buffer_at
  %head = load ptr, ptr @__cyborg_writers
  store ptr %head, ptr %next_at
  %has_head = icmp ne ptr %head, null
  br i1 %has_head, label %link_head, label %linked
link_head:
  %head_previous_at = getelementptr %writer, ptr %head, i32 0, i32 5
  store ptr %writer, ptr %head_previous_at
  br label %linked
linked:
  store ptr %writer, ptr @__cyborg_writers
  ret void
unbuffered:
  store ptr null, ptr %buffer_at
  store ptr null, ptr %next_at
  ret void
}

define void @__cyborg_writer_flush(ptr %writer) {
entry:
  %length_at = getelementptr %writer, ptr %writer, i32 0, i32 3
  %length = load i64, ptr %length_at
  %empty = icmp eq i64 %length, 0
  br i1 %empty, label %exit, label %flush
flush:
  %descriptor_at = getelementptr %writer, ptr %writer, i32 0, i32 0
  %descriptor = load i32, ptr %descriptor_at
  %buffer_at = getelementptr %writer, ptr %writer, i32 0, i32 1
  %buffer = load ptr, ptr %buffer_at
  call void @__cyborg_write_all(i32 %descriptor, ptr %buffer, i64 %length)
  store i64 0, ptr %length_at
  br label %exit
exit:
  ret void
}

define void @__cyborg_writer_write(ptr %writer, ptr %data, i64 %length) {
entry:
  %capacity_at = getelementptr %writer, ptr %writer, i32 0, i32 2
  %capacity = load i64, ptr %capacity_at
  %length_at = getelementptr %writer, ptr %writer, i32 0, i32 3
  %held = load i64, ptr %length_at
  %together = add i64 %held, %length
  %fits = icmp sle i64 %together, %capacity
  br i1 %fits, label %copy, label %full
full:
  call void @__cyborg_writer_flush(ptr %writer)
  %fits_alone = icmp slt i64 %length, %capacity
  br i1 %fits_alone, label %copy, label %direct
direct:
  %descriptor_at = getelementptr %writer, ptr %writer, i32 0, i32 0
  %descriptor = load i32, ptr %descriptor_at
  call void @__cyborg_write_all(i32 %descriptor, ptr %data, i64 %length)
  ret void
copy:
  %now = load i64, ptr %length_at
  %buffer_at = getelementptr %writer, ptr %writer, i32 0, i32 1
  %buffer = load ptr, ptr %buffer_at
  %end = getelementptr i8, ptr %buffer, i64 %now
  call void @llvm.memcpy.p0.p0.i64(ptr %end, ptr %data, i64 %length, i1 false)
  %after = add i64 %now, %length
  store i64 %after, ptr %length_at
  ret void
}

define void @__cyborg_writer_done(ptr %writer) {
entry:
  call void @__cyborg_writer_flush(ptr %writer)
  %buffer_at = getelementptr %writer, ptr %writer, i32 0, i32 1
  %buffer = load ptr, ptr %buffer_at
  %buffered = icmp ne ptr %buffer, null
  br i1 %buffered, label %unlink, label %exit
unlink:
  %next_at = getelementptr %writer, ptr %writer, i32 0, i32 4
  %next = load ptr, ptr %next_at
  %previous_at = getelementptr %writer, ptr %writer, i32 0, i32 5
  %previous = load ptr, ptr %previous_at
  %has_next = icmp ne ptr %next, null
  br i1 %has_next, label %fix_next, label %fixed_next
fix_next:
  %next_previous_at = getelementptr %writer, ptr %next, i32 0, i32 5
  store ptr %previous, ptr %next_previous_at
  br label %fixed_next
fixed_next:
  %has_previous = icmp ne ptr %previous, null
  br i1 %has_previous, label %fix_previous, label %fix_head
fix_previous:
  %previous_next_at = getelementptr %writer, ptr %previous, i32 0, i32 4
  store ptr %next, ptr %previous_next_at
  br label %freed
fix_head:
  store ptr %next, ptr @__cyborg_writers
  br label %freed
freed:
  call void @free(ptr %buffer)
  store ptr null, ptr %buffer_at
  br label %exit
exit:
  ret void
}

; Before a crash's message, every buffered writer still holding output is flushed, so the last thing printed always
; shows (D274).
define void @__cyborg_flush_all() {
entry:
  %head = load ptr, ptr @__cyborg_writers
  br label %loop
loop:
  %writer = phi ptr [ %head, %entry ], [ %next, %flush ]
  %finished = icmp eq ptr %writer, null
  br i1 %finished, label %exit, label %flush
flush:
  call void @__cyborg_writer_flush(ptr %writer)
  %next_at = getelementptr %writer, ptr %writer, i32 0, i32 4
  %next = load ptr, ptr %next_at
  br label %loop
exit:
  ret void
}

; Starts a crash: flushes every writer, and opens an unbuffered writer on stderr for the message.
define void @__cyborg_crash_begin(ptr %message_writer) {
entry:
  call void @__cyborg_flush_all()
  call void @__cyborg_writer_open(ptr %message_writer, i32 2, i64 0)
  ret void
}

define void @__cyborg_exit(i32 %code) noreturn {
entry:
  call void @exit(i32 %code)
  unreachable
}

; Prints an integer: its magnitude, whether it's negative, its base, upper-case hex digits, a width in columns with
; its alignment (0 left, 1 right, 2 center), zeros to fill the width, and a sign even when positive.
define void @__cyborg_print_integer(ptr %writer, i64 %magnitude, i1 %negative, i64 %base, i1 %upper, i64 %width, i32 %align, i1 %zeros, i1 %sign) {
entry:
  %digits = alloca [72 x i8]
  %end = getelementptr i8, ptr %digits, i64 72
  br label %digit
digit:
  %value = phi i64 [ %magnitude, %entry ], [ %rest, %digit ]
  %at = phi ptr [ %end, %entry ], [ %here, %digit ]
  %rest = udiv i64 %value, %base
  %remainder = urem i64 %value, %base
  %small = icmp ult i64 %remainder, 10
  %letter_base = select i1 %upper, i64 55, i64 87
  %offset = select i1 %small, i64 48, i64 %letter_base
  %code = add i64 %remainder, %offset
  %byte = trunc i64 %code to i8
  %here = getelementptr i8, ptr %at, i64 -1
  store i8 %byte, ptr %here
  %more = icmp ne i64 %rest, 0
  br i1 %more, label %digit, label %signed
signed:
  %start_int = ptrtoint ptr %here to i64
  %end_int = ptrtoint ptr %end to i64
  %count = sub i64 %end_int, %start_int
  %minus = select i1 %negative, i8 45, i8 43
  %shows_sign = or i1 %negative, %sign
  %sign_width = zext i1 %shows_sign to i64
  %total = add i64 %count, %sign_width
  %pad = sub i64 %width, %total
  %padded = icmp sgt i64 %pad, 0
  %padding = select i1 %padded, i64 %pad, i64 0
  br i1 %zeros, label %zero_fill, label %space_fill
zero_fill:
  br i1 %shows_sign, label %zero_sign, label %zero_digits
zero_sign:
  %sign_cell = alloca i8
  store i8 %minus, ptr %sign_cell
  call void @__cyborg_writer_write(ptr %writer, ptr %sign_cell, i64 1)
  br label %zero_digits
zero_digits:
  call void @__cyborg_print_repeat(ptr %writer, i8 48, i64 %padding)
  call void @__cyborg_writer_write(ptr %writer, ptr %here, i64 %count)
  ret void
space_fill:
  %is_left = icmp eq i32 %align, 0
  %is_center = icmp eq i32 %align, 2
  %half = lshr i64 %padding, 1
  %center_before = select i1 %is_center, i64 %half, i64 %padding
  %before = select i1 %is_left, i64 0, i64 %center_before
  %after = sub i64 %padding, %before
  call void @__cyborg_print_repeat(ptr %writer, i8 32, i64 %before)
  br i1 %shows_sign, label %space_sign, label %space_digits
space_sign:
  %space_sign_cell = alloca i8
  store i8 %minus, ptr %space_sign_cell
  call void @__cyborg_writer_write(ptr %writer, ptr %space_sign_cell, i64 1)
  br label %space_digits
space_digits:
  call void @__cyborg_writer_write(ptr %writer, ptr %here, i64 %count)
  call void @__cyborg_print_repeat(ptr %writer, i8 32, i64 %after)
  ret void
}

; Prints a byte `count` times.
define void @__cyborg_print_repeat(ptr %writer, i8 %byte, i64 %count) {
entry:
  %cell = alloca i8
  store i8 %byte, ptr %cell
  br label %loop
loop:
  %left = phi i64 [ %count, %entry ], [ %next, %write ]
  %finished = icmp sle i64 %left, 0
  br i1 %finished, label %exit, label %write
write:
  call void @__cyborg_writer_write(ptr %writer, ptr %cell, i64 1)
  %next = sub i64 %left, 1
  br label %loop
exit:
  ret void
}

; Prints text, padded to a width in columns, aligned 0 left, 1 right or 2 center.
define void @__cyborg_print_text(ptr %writer, ptr %data, i64 %length, i64 %columns, i64 %width, i32 %align) {
entry:
  %pad = sub i64 %width, %columns
  %padded = icmp sgt i64 %pad, 0
  %padding = select i1 %padded, i64 %pad, i64 0
  %is_left = icmp eq i32 %align, 0
  %is_center = icmp eq i32 %align, 2
  %half = lshr i64 %padding, 1
  %center_before = select i1 %is_center, i64 %half, i64 %padding
  %before = select i1 %is_left, i64 0, i64 %center_before
  %after = sub i64 %padding, %before
  call void @__cyborg_print_repeat(ptr %writer, i8 32, i64 %before)
  call void @__cyborg_writer_write(ptr %writer, ptr %data, i64 %length)
  call void @__cyborg_print_repeat(ptr %writer, i8 32, i64 %after)
  ret void
}
