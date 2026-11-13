; The runtime for a program on an operating system: writers (D272, D273), crashing (D96, D274) and printing (D194).
; It's LLVM IR, read through Lyftar (rule 1), and linked into each program before optimizing, so its small functions
; inline into the code that calls them. The C library's `write`, `malloc`, `free` and `exit` come from libSystem on
; macOS and from musl on Linux (D261).

; A writer: its file descriptor, its buffer, the buffer's size and how much it holds, and its neighbours in the list
; of buffered writers, which a crash flushes before its message (D274). An unbuffered writer has no buffer.
%writer = type { i32, ptr, i64, i64, ptr, ptr }

@__cyborg_writers = hidden global ptr null

; The runtime's own lock, a lock word like a `@concur::lock`'s, held while the list of writers or an allocator's
; record changes, since tasks on the OS's threads share them (D345).
@__cyborg_runtime_lock = hidden global i32 0

declare i64 @write(i32, ptr, i64)
declare ptr @malloc(i64)
declare ptr @realloc(ptr, i64)
declare void @free(ptr)
declare void @exit(i32) noreturn

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
  call void @__cyborg_lock(ptr @__cyborg_runtime_lock)
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
  call void @__cyborg_unlock(ptr @__cyborg_runtime_lock)
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
  call void @__cyborg_lock(ptr @__cyborg_runtime_lock)
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
  call void @__cyborg_unlock(ptr @__cyborg_runtime_lock)
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
  call void @__cyborg_lock(ptr @__cyborg_runtime_lock)
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
  call void @__cyborg_unlock(ptr @__cyborg_runtime_lock)
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

; Prints formatted text padded to a width, aligned 0 left, 1 right or 2 center. With %zeros, zeros fill the width
; after any sign the text starts with, as a number's do.
define void @__cyborg_print_padded(ptr %writer, ptr %text, i64 %length, i64 %width, i32 %align, i1 %zeros) {
entry:
  br i1 %zeros, label %zero_fill, label %spaces
spaces:
  call void @__cyborg_print_text(ptr %writer, ptr %text, i64 %length, i64 %length, i64 %width, i32 %align)
  ret void
zero_fill:
  %pad = sub i64 %width, %length
  %padded = icmp sgt i64 %pad, 0
  %padding = select i1 %padded, i64 %pad, i64 0
  %first = load i8, ptr %text
  %is_minus = icmp eq i8 %first, 45
  %is_plus = icmp eq i8 %first, 43
  %has_sign = or i1 %is_minus, %is_plus
  br i1 %has_sign, label %sign, label %digits
sign:
  call void @__cyborg_writer_write(ptr %writer, ptr %text, i64 1)
  br label %digits
digits:
  %skip = zext i1 %has_sign to i64
  %rest = getelementptr i8, ptr %text, i64 %skip
  %rest_length = sub i64 %length, %skip
  call void @__cyborg_print_repeat(ptr %writer, i8 48, i64 %padding)
  call void @__cyborg_writer_write(ptr %writer, ptr %rest, i64 %rest_length)
  ret void
}

; Heap text from other text (D326, D331, D369): its bytes copied into memory of its own, which `str.done` frees.
define { ptr, i64 } @__cyborg_text_copy(ptr %data, i64 %length) {
entry:
  %room = icmp eq i64 %length, 0
  %bytes = select i1 %room, i64 1, i64 %length
  %text = call ptr @malloc(i64 %bytes)
  %none = icmp eq ptr %text, null
  br i1 %none, label %out, label %fine
out:
  call void @exit(i32 1)
  unreachable
fine:
  call void @llvm.memcpy.p0.p0.i64(ptr %text, ptr %data, i64 %length, i1 false)
  %with_data = insertvalue { ptr, i64 } undef, ptr %text, 0
  %result = insertvalue { ptr, i64 } %with_data, i64 %length, 1
  ret { ptr, i64 } %result
}

define void @__cyborg_text_free({ ptr, i64 } %text) {
entry:
  %data = extractvalue { ptr, i64 } %text, 0
  call void @free(ptr %data)
  ret void
}

; `as str` on a number builds new text, which can allocate (D7, D122). It's never freed yet.
define { ptr, i64 } @__cyborg_text_of_integer(i64 %magnitude, i1 %negative) {
entry:
  %text = call ptr @malloc(i64 24)
  %writer = alloca %writer
  ; Write the digits through a buffer, then copy them in.
  %digits = alloca [24 x i8]
  %end = getelementptr i8, ptr %digits, i64 24
  br label %digit
digit:
  %value = phi i64 [ %magnitude, %entry ], [ %rest, %digit ]
  %at = phi ptr [ %end, %entry ], [ %here, %digit ]
  %rest = udiv i64 %value, 10
  %remainder = urem i64 %value, 10
  %code = add i64 %remainder, 48
  %byte = trunc i64 %code to i8
  %here = getelementptr i8, ptr %at, i64 -1
  store i8 %byte, ptr %here
  %more = icmp ne i64 %rest, 0
  br i1 %more, label %digit, label %signed
signed:
  %minus_at = getelementptr i8, ptr %here, i64 -1
  store i8 45, ptr %minus_at
  %start = select i1 %negative, ptr %minus_at, ptr %here
  %start_int = ptrtoint ptr %start to i64
  %end_int = ptrtoint ptr %end to i64
  %length = sub i64 %end_int, %start_int
  call void @llvm.memcpy.p0.p0.i64(ptr %text, ptr %start, i64 %length, i1 false)
  %with_data = insertvalue { ptr, i64 } undef, ptr %text, 0
  %result = insertvalue { ptr, i64 } %with_data, i64 %length, 1
  ret { ptr, i64 } %result
}

define { ptr, i64 } @__cyborg_text_of_float(double %value, i32 %bits) {
entry:
  %buffer = alloca [1400 x i8]
  %length = call i64 @__cyborg_format_float(double %value, i32 %bits, i32 0, i32 0, i1 false, ptr %buffer)
  %text = call ptr @malloc(i64 %length)
  call void @llvm.memcpy.p0.p0.i64(ptr %text, ptr %buffer, i64 %length, i1 false)
  %with_data = insertvalue { ptr, i64 } undef, ptr %text, 0
  %result = insertvalue { ptr, i64 } %with_data, i64 %length, 1
  ret { ptr, i64 } %result
}

; Lists (D11, D24): a record of where the items are, how many there are, how many fit before it grows, which
; allocator its items came from, or -1 before it has any (D322), and where its block starts in an arena, or -1.
%list = type { ptr, i64, i64, i64, i64 }

; An allocator (D321, D324, D330): its number, the room it has, or -1 for the OS heap, which has no limit of its own,
; how much is taken, where its last block starts, and how many lists still hold a block from it. Every `os::heap()` is
; the one heap, number 0 (D371), and each arena gets the next number. An arena counts its room in the standard's
; sizes, so it runs out at the same byte on every machine for a target (D370, D389); the items themselves live in
; memory from the OS heap.
%allocator = type { i64, i64, i64, i64, i64 }

@__cyborg_the_heap = hidden global %allocator { i64 0, i64 -1, i64 0, i64 -1, i64 0 }
@__cyborg_next_allocator = hidden global i64 1

define ptr @__cyborg_heap() {
entry:
  ret ptr @__cyborg_the_heap
}

; A new arena, with its room in bytes.
define ptr @__cyborg_arena_new(i64 %room) {
entry:
  %record = call ptr @malloc(i64 40)
  %none = icmp eq ptr %record, null
  br i1 %none, label %out, label %fine
out:
  call void @exit(i32 1)
  unreachable
fine:
  %id = atomicrmw add ptr @__cyborg_next_allocator, i64 1 monotonic
  store %allocator { i64 0, i64 0, i64 0, i64 -1, i64 0 }, ptr %record
  %id_place = getelementptr %allocator, ptr %record, i32 0, i32 0
  store i64 %id, ptr %id_place
  %room_place = getelementptr %allocator, ptr %record, i32 0, i32 1
  store i64 %room, ptr %room_place
  ret ptr %record
}

; How many lists still hold a block from an allocator.
define i64 @__cyborg_alive(ptr %allocator) {
entry:
  %alive_place = getelementptr %allocator, ptr %allocator, i32 0, i32 4
  %alive = load i64, ptr %alive_place
  ret i64 %alive
}

define void @__cyborg_arena_free(ptr %arena) {
entry:
  call void @free(ptr %arena)
  ret void
}

; Whether a list is handed the allocator it uses: 0 if so, or 2 with its allocator's number and this one's in
; `%report` (D322).
define i64 @__cyborg_list_same_allocator(ptr %list, ptr %allocator, ptr %report) {
entry:
  %id_place = getelementptr %list, ptr %list, i32 0, i32 3
  %id = load i64, ptr %id_place
  %given_place = getelementptr %allocator, ptr %allocator, i32 0, i32 0
  %given = load i64, ptr %given_place
  %has = icmp sge i64 %id, 0
  %other = icmp ne i64 %id, %given
  %wrong = and i1 %has, %other
  br i1 %wrong, label %refuse, label %fine
refuse:
  store i64 %id, ptr %report
  %second = getelementptr i64, ptr %report, i64 1
  store i64 %given, ptr %second
  ret i64 2
fine:
  ret i64 0
}

; Makes room in a list for `%want` items, growing as its type says (D385): `%grow` is 0 to double, from room for at
; least 4, 1 for exactly what's needed, 2 for steps of `%step` items, and 3 for steps of `%step` bytes. Gives 0 when
; there's room, 1 when the allocator is out of memory, with its room, what's taken, what's needed and from which
; byte in `%report`, and 2 for another allocator than the list's (D322). `%size` is an item's size in this
; machine's memory, and `%standard` and `%align` are its size and alignment as the standard counts them (D370).
define i64 @__cyborg_list_reserve(ptr %list, ptr %allocator, i64 %want, i64 %size, i64 %standard, i64 %align, i64 %grow, i64 %step, ptr %report) {
entry:
  call void @__cyborg_lock(ptr @__cyborg_runtime_lock)
  %result = call i64 @__cyborg_list_reserve_held(ptr %list, ptr %allocator, i64 %want, i64 %size, i64 %standard, i64 %align, i64 %grow, i64 %step, ptr %report)
  call void @__cyborg_unlock(ptr @__cyborg_runtime_lock)
  ret i64 %result
}

; `__cyborg_list_reserve` with the runtime's lock held, since the allocator's record changes.
define i64 @__cyborg_list_reserve_held(ptr %list, ptr %allocator, i64 %want, i64 %size, i64 %standard, i64 %align, i64 %grow, i64 %step, ptr %report) {
entry:
  %same = call i64 @__cyborg_list_same_allocator(ptr %list, ptr %allocator, ptr %report)
  %wrong = icmp ne i64 %same, 0
  br i1 %wrong, label %refuse, label %checked
refuse:
  ret i64 2
checked:
  %cap_place = getelementptr %list, ptr %list, i32 0, i32 2
  %cap = load i64, ptr %cap_place
  %enough = icmp uge i64 %cap, %want
  br i1 %enough, label %done, label %grow_it
done:
  ret i64 0
grow_it:
  ; Doubling, from at least 4.
  %empty = icmp eq i64 %cap, 0
  %doubled = shl i64 %cap, 1
  %at_least_four = select i1 %empty, i64 4, i64 %doubled
  %double_small = icmp ult i64 %at_least_four, %want
  %double_cap = select i1 %double_small, i64 %want, i64 %at_least_four
  ; Steps of items.
  %step_less = sub i64 %step, 1
  %items_up = add i64 %want, %step_less
  %items_steps = udiv i64 %items_up, %step
  %items_cap = mul i64 %items_steps, %step
  ; Steps of bytes.
  %want_bytes = mul i64 %want, %standard
  %bytes_up = add i64 %want_bytes, %step_less
  %bytes_steps = udiv i64 %bytes_up, %step
  %step_bytes = mul i64 %bytes_steps, %step
  %bytes_cap = udiv i64 %step_bytes, %standard
  %is_exact = icmp eq i64 %grow, 1
  %is_items = icmp eq i64 %grow, 2
  %is_bytes = icmp eq i64 %grow, 3
  %cap_a = select i1 %is_exact, i64 %want, i64 %double_cap
  %cap_b = select i1 %is_items, i64 %items_cap, i64 %cap_a
  %new_cap = select i1 %is_bytes, i64 %bytes_cap, i64 %cap_b
  %counted = mul i64 %new_cap, %standard
  %standard_bytes = select i1 %is_bytes, i64 %step_bytes, i64 %counted
  %room_place = getelementptr %allocator, ptr %allocator, i32 0, i32 1
  %room = load i64, ptr %room_place
  %unlimited = icmp slt i64 %room, 0
  br i1 %unlimited, label %memory, label %count
count:
  %block_place = getelementptr %list, ptr %list, i32 0, i32 4
  %block = load i64, ptr %block_place
  %taken_place = getelementptr %allocator, ptr %allocator, i32 0, i32 2
  %taken = load i64, ptr %taken_place
  %last_place = getelementptr %allocator, ptr %allocator, i32 0, i32 3
  %last = load i64, ptr %last_place
  %has_block = icmp sge i64 %block, 0
  %is_last = icmp eq i64 %block, %last
  %in_place = and i1 %has_block, %is_last
  ; A block grows where it is while it's the allocator's last one, and otherwise a new one is taken (D389).
  %align_less = sub i64 %align, 1
  %taken_up = add i64 %taken, %align_less
  %aligned_steps = udiv i64 %taken_up, %align
  %aligned = mul i64 %aligned_steps, %align
  %start = select i1 %in_place, i64 %block, i64 %aligned
  %end = add i64 %start, %standard_bytes
  %too_much = icmp sgt i64 %end, %room
  br i1 %too_much, label %out, label %take
out:
  store i64 %room, ptr %report
  %report_taken = getelementptr i64, ptr %report, i64 1
  store i64 %taken, ptr %report_taken
  %report_needs = getelementptr i64, ptr %report, i64 2
  store i64 %standard_bytes, ptr %report_needs
  %report_from = getelementptr i64, ptr %report, i64 3
  store i64 %start, ptr %report_from
  ret i64 1
take:
  store i64 %end, ptr %taken_place
  store i64 %start, ptr %last_place
  store i64 %start, ptr %block_place
  br i1 %has_block, label %memory, label %first
first:
  %alive_place = getelementptr %allocator, ptr %allocator, i32 0, i32 4
  %alive = load i64, ptr %alive_place
  %more_alive = add i64 %alive, 1
  store i64 %more_alive, ptr %alive_place
  br label %memory
memory:
  %bytes = mul i64 %new_cap, %size
  %storage_place = getelementptr %list, ptr %list, i32 0, i32 0
  %old = load ptr, ptr %storage_place
  %new = call ptr @realloc(ptr %old, i64 %bytes)
  %none = icmp eq ptr %new, null
  br i1 %none, label %no_memory, label %grown
no_memory:
  store i64 0, ptr %report
  %none_taken = getelementptr i64, ptr %report, i64 1
  store i64 0, ptr %none_taken
  %none_needs = getelementptr i64, ptr %report, i64 2
  store i64 %bytes, ptr %none_needs
  %none_from = getelementptr i64, ptr %report, i64 3
  store i64 0, ptr %none_from
  ret i64 1
grown:
  store ptr %new, ptr %storage_place
  store i64 %new_cap, ptr %cap_place
  %given_place = getelementptr %allocator, ptr %allocator, i32 0, i32 0
  %given = load i64, ptr %given_place
  %id_place = getelementptr %list, ptr %list, i32 0, i32 3
  store i64 %given, ptr %id_place
  ret i64 0
}

; Ends a list (D323, D331): its memory goes back, and an arena counts one fewer alive. The allocator was checked first.
define void @__cyborg_list_done(ptr %list, ptr %allocator) {
entry:
  %block_place = getelementptr %list, ptr %list, i32 0, i32 4
  %block = load i64, ptr %block_place
  %has_block = icmp sge i64 %block, 0
  br i1 %has_block, label %count, label %free
count:
  %alive_place = getelementptr %allocator, ptr %allocator, i32 0, i32 4
  %alive = atomicrmw sub ptr %alive_place, i64 1 monotonic
  br label %free
free:
  %storage_place = getelementptr %list, ptr %list, i32 0, i32 0
  %storage = load ptr, ptr %storage_place
  call void @free(ptr %storage)
  ret void
}

; Memory for a list's first items. Running out of memory stops the program, since nothing can carry on without it.
define ptr @__cyborg_allocate(i64 %bytes) {
entry:
  %memory = call ptr @malloc(i64 %bytes)
  %none = icmp eq ptr %memory, null
  br i1 %none, label %out, label %fine
out:
  call void @exit(i32 1)
  unreachable
fine:
  ret ptr %memory
}

; Makes room for one more item at the end of a list, doubling its room when it's full, and gives the new item's place.
define ptr @__cyborg_list_push(ptr %list, i64 %item_size) {
entry:
  %storage_place = getelementptr %list, ptr %list, i32 0, i32 0
  %count_place = getelementptr %list, ptr %list, i32 0, i32 1
  %room_place = getelementptr %list, ptr %list, i32 0, i32 2
  %count = load i64, ptr %count_place
  %room = load i64, ptr %room_place
  %full = icmp uge i64 %count, %room
  br i1 %full, label %grow, label %place
grow:
  %doubled = shl i64 %room, 1
  %small = icmp ult i64 %doubled, 4
  %new_room = select i1 %small, i64 4, i64 %doubled
  %bytes = mul i64 %new_room, %item_size
  %old = load ptr, ptr %storage_place
  %new = call ptr @realloc(ptr %old, i64 %bytes)
  %none = icmp eq ptr %new, null
  br i1 %none, label %out, label %grown
out:
  call void @exit(i32 1)
  unreachable
grown:
  store ptr %new, ptr %storage_place
  store i64 %new_room, ptr %room_place
  br label %place
place:
  %storage = load ptr, ptr %storage_place
  %offset = mul i64 %count, %item_size
  %slot = getelementptr i8, ptr %storage, i64 %offset
  %more = add i64 %count, 1
  store i64 %more, ptr %count_place
  ret ptr %slot
}

; Locks (D348): a lock's word is 0 while it's free. `__cyborg_lock` and `__cyborg_unlock` come from the system's own
; part of the runtime, and put a waiting task to sleep rather than spinning, since a lock can be held a long time.

; Sorts an array of addresses into increasing order, in place. It's short, so insertion sort.
define void @__cyborg_sort_addresses(ptr %words, i64 %count) {
entry:
  br label %outer
outer:
  %i = phi i64 [ 1, %entry ], [ %i_next, %placed ]
  %more = icmp ult i64 %i, %count
  br i1 %more, label %pick, label %done
pick:
  %at = getelementptr ptr, ptr %words, i64 %i
  %item = load ptr, ptr %at
  %key = ptrtoint ptr %item to i64
  br label %inner
inner:
  %j = phi i64 [ %i, %pick ], [ %j_less, %shift ]
  %has_left = icmp ugt i64 %j, 0
  br i1 %has_left, label %look, label %place
look:
  %j_less = sub i64 %j, 1
  %left_at = getelementptr ptr, ptr %words, i64 %j_less
  %left = load ptr, ptr %left_at
  %left_key = ptrtoint ptr %left to i64
  %bigger = icmp ugt i64 %left_key, %key
  br i1 %bigger, label %shift, label %place
shift:
  %here_at = getelementptr ptr, ptr %words, i64 %j
  store ptr %left, ptr %here_at
  br label %inner
place:
  %slot_at = getelementptr ptr, ptr %words, i64 %j
  store ptr %item, ptr %slot_at
  br label %placed
placed:
  %i_next = add i64 %i, 1
  br label %outer
done:
  ret void
}

; Takes every lock whose word's address is in the array, in address order, so blocks taking the same locks always
; take them in the same order, and can't each hold one the other waits for (D349). One lock reached twice is taken
; once. The array is left sorted, for `__cyborg_unlock_all`.
define void @__cyborg_lock_all(ptr %words, i64 %count) {
entry:
  call void @__cyborg_sort_addresses(ptr %words, i64 %count)
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %next, %step ]
  %more = icmp ult i64 %i, %count
  br i1 %more, label %look, label %done
look:
  %at = getelementptr ptr, ptr %words, i64 %i
  %word = load ptr, ptr %at
  %first = icmp eq i64 %i, 0
  br i1 %first, label %take, label %compare
compare:
  %before_index = sub i64 %i, 1
  %before_at = getelementptr ptr, ptr %words, i64 %before_index
  %before = load ptr, ptr %before_at
  %same = icmp eq ptr %before, %word
  br i1 %same, label %step, label %take
take:
  call void @__cyborg_lock(ptr %word)
  br label %step
step:
  %next = add i64 %i, 1
  br label %loop
done:
  ret void
}

; Gives back the locks `__cyborg_lock_all` took from the same array, which it left sorted.
define void @__cyborg_unlock_all(ptr %words, i64 %count) {
entry:
  br label %loop
loop:
  %i = phi i64 [ 0, %entry ], [ %next, %step ]
  %more = icmp ult i64 %i, %count
  br i1 %more, label %look, label %done
look:
  %at = getelementptr ptr, ptr %words, i64 %i
  %word = load ptr, ptr %at
  %first = icmp eq i64 %i, 0
  br i1 %first, label %give, label %compare
compare:
  %before_index = sub i64 %i, 1
  %before_at = getelementptr ptr, ptr %words, i64 %before_index
  %before = load ptr, ptr %before_at
  %same = icmp eq ptr %before, %word
  br i1 %same, label %step, label %give
give:
  call void @__cyborg_unlock(ptr %word)
  br label %step
step:
  %next = add i64 %i, 1
  br label %loop
done:
  ret void
}

; Runners from the OS (D345, D392). A pool holds how many of its tasks may run at once, how many run now, and the
; lock and condition a spawn waits on while every thread is busy. 64 bytes is room for either system's
; `pthread_mutex_t` and `pthread_cond_t`.
%pool = type { i64, i64, [64 x i8], [64 x i8] }

declare i32 @pthread_create(ptr, ptr, ptr, ptr)
declare i32 @pthread_join(ptr, ptr)
declare i32 @pthread_mutex_init(ptr, ptr)
declare i32 @pthread_mutex_lock(ptr)
declare i32 @pthread_mutex_unlock(ptr)
declare i32 @pthread_mutex_destroy(ptr)
declare i32 @pthread_cond_init(ptr, ptr)
declare i32 @pthread_cond_wait(ptr, ptr)
declare i32 @pthread_cond_signal(ptr)
declare i32 @pthread_cond_destroy(ptr)
declare i64 @sysconf(i32)

; A pool of `%count` threads, or one for every core online when `%count` is 0, for `count: max`, asking the system
; through `sysconf` with its own number for that question, `%cores_name`.
define ptr @__cyborg_pool_new(i64 %count, i32 %cores_name) {
entry:
  %every = icmp eq i64 %count, 0
  br i1 %every, label %cores, label %make
cores:
  %online = call i64 @sysconf(i32 %cores_name)
  %some = icmp sgt i64 %online, 0
  %known = select i1 %some, i64 %online, i64 1
  br label %make
make:
  %threads = phi i64 [ %count, %entry ], [ %known, %cores ]
  %pool = call ptr @__cyborg_allocate(i64 144)
  %count_at = getelementptr %pool, ptr %pool, i32 0, i32 0
  store i64 %threads, ptr %count_at
  %running_at = getelementptr %pool, ptr %pool, i32 0, i32 1
  store i64 0, ptr %running_at
  %mutex = getelementptr %pool, ptr %pool, i32 0, i32 2
  %mutex_made = call i32 @pthread_mutex_init(ptr %mutex, ptr null)
  %condition = getelementptr %pool, ptr %pool, i32 0, i32 3
  %condition_made = call i32 @pthread_cond_init(ptr %condition, ptr null)
  ret ptr %pool
}

; Ends a pool, whose tasks are all joined by now (D343, D392).
define void @__cyborg_pool_done(ptr %pool) {
entry:
  %mutex = getelementptr %pool, ptr %pool, i32 0, i32 2
  %mutex_gone = call i32 @pthread_mutex_destroy(ptr %mutex)
  %condition = getelementptr %pool, ptr %pool, i32 0, i32 3
  %condition_gone = call i32 @pthread_cond_destroy(ptr %condition)
  call void @free(ptr %pool)
  ret void
}

; A task's record starts with its thread and its pool, then what its call gives back and its arguments, which the
; compiler lays out per call. `%run` is the function the thread runs: it unpacks the arguments, makes the call,
; stores what it gave back, and calls `__cyborg_task_end`.
;
; Starts a task on one of the pool's threads, waiting while they're all busy (D345). If the system can't start a
; thread, the task runs here instead, so it still runs and its join finds it done.
define void @__cyborg_task_start(ptr %task, ptr %run) {
entry:
  %pool_at = getelementptr { ptr, ptr }, ptr %task, i32 0, i32 1
  %pool = load ptr, ptr %pool_at
  %count_at = getelementptr %pool, ptr %pool, i32 0, i32 0
  %running_at = getelementptr %pool, ptr %pool, i32 0, i32 1
  %mutex = getelementptr %pool, ptr %pool, i32 0, i32 2
  %condition = getelementptr %pool, ptr %pool, i32 0, i32 3
  %locked = call i32 @pthread_mutex_lock(ptr %mutex)
  br label %check
check:
  %running = load i64, ptr %running_at
  %count = load i64, ptr %count_at
  %full = icmp sge i64 %running, %count
  br i1 %full, label %wait, label %claim
wait:
  %woken = call i32 @pthread_cond_wait(ptr %condition, ptr %mutex)
  br label %check
claim:
  %more = add i64 %running, 1
  store i64 %more, ptr %running_at
  %unlocked = call i32 @pthread_mutex_unlock(ptr %mutex)
  %made = call i32 @pthread_create(ptr %task, ptr null, ptr %run, ptr %task)
  %failed = icmp ne i32 %made, 0
  br i1 %failed, label %here, label %done
here:
  store ptr null, ptr %task
  %ran = call ptr %run(ptr %task)
  br label %done
done:
  ret void
}

; A task's call has finished: its thread is free for the next spawn waiting.
define void @__cyborg_task_end(ptr %task) {
entry:
  %pool_at = getelementptr { ptr, ptr }, ptr %task, i32 0, i32 1
  %pool = load ptr, ptr %pool_at
  %running_at = getelementptr %pool, ptr %pool, i32 0, i32 1
  %mutex = getelementptr %pool, ptr %pool, i32 0, i32 2
  %condition = getelementptr %pool, ptr %pool, i32 0, i32 3
  %locked = call i32 @pthread_mutex_lock(ptr %mutex)
  %running = load i64, ptr %running_at
  %fewer = sub i64 %running, 1
  store i64 %fewer, ptr %running_at
  %signalled = call i32 @pthread_cond_signal(ptr %condition)
  %unlocked = call i32 @pthread_mutex_unlock(ptr %mutex)
  ret void
}

; Waits for a task's thread to finish (D343), after which what it stored can be read.
define void @__cyborg_task_join(ptr %task) {
entry:
  %thread = load ptr, ptr %task
  %ran_here = icmp eq ptr %thread, null
  br i1 %ran_here, label %done, label %join
join:
  %joined = call i32 @pthread_join(ptr %thread, ptr null)
  br label %done
done:
  ret void
}

define void @__cyborg_task_free(ptr %task) {
entry:
  call void @free(ptr %task)
  ret void
}

; Stops a program tuned with `--machine.cpu=` on a machine missing instructions it was built to use, with the
; message the compiler wrote for that instruction family, instead of crashing midway (D365, D420).
define void @__cyborg_machine_missing(ptr %message, i64 %length) noreturn {
entry:
  call void @__cyborg_flush_all()
  call void @__cyborg_write_all(i32 2, ptr %message, i64 %length)
  call void @exit(i32 1)
  unreachable
}
