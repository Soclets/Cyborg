// trace: D49, D116
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to 6;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a bit_and 3));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a bit_or 1));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a bit_xor 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (bit_not $a));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a shl 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a shr 1));
    let $u: u8 set_to 15;
    let $v: u8 set_to bit_not $u;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $v);
    let $w: i32 set_to ($a shl 2) + 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $w);
    return void;
end fn.main
