// An array's `$a.len` follows the same rule, which is Claude's reading in D140.
// trace: D140
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: std::array<i32>[len: 2 items, len.type: u8] = [1, 2];
    let $n: u8[unit: items] = $a.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
