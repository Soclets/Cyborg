// trace: D49, D116
import std;

fn.main [none] -> void:
    let $a: i32 = 6;
    std::io::fmt_print("{}{newln}", ($a bit_and 3));
    std::io::fmt_print("{}{newln}", ($a bit_or 1));
    std::io::fmt_print("{}{newln}", ($a bit_xor 2));
    std::io::fmt_print("{}{newln}", (bit_not $a));
    std::io::fmt_print("{}{newln}", ($a shl 2));
    std::io::fmt_print("{}{newln}", ($a shr 1));
    let $u: u8 = 15;
    let $v: u8 = bit_not $u;
    std::io::fmt_print("{}{newln}", $v);
    let $w: i32 = ($a shl 2) + 1;
    std::io::fmt_print("{}{newln}", $w);
    return void;
end fn.main
