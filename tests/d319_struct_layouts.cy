// A bare `struct` keeps the written order, `struct.any_order` lets the compiler reorder, and
// `struct.packed` has no padding. Reading a packed field by name is fine.
// trace: D319, D320
import std;
import os;

struct.any_order cache_entry:
    key: u64;
    hits: u32;
end struct.any_order cache_entry

struct.packed header:
    kind: u8;
    length: u32;
end struct.packed header

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $e: cache_entry = cache_entry[key = 7, hits = 3];
    let $h: header = header[kind = 1, length = 512];
    let $length: u32 = $h.length;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $e.key, $e.hits, $h.kind, $length);
    return void;
end fn.main
