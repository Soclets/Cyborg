// trace: D49, D116, D361
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 6;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a and.bit 3));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a or.bit 1));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a xor.bit 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (not.bit $a));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a shl 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a shr 1));
    let $u: u8 = 15;
    let $v: u8 = not.bit $u;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $v);
    let $w: i32 = ($a shl 2) + 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $w);
    return void;
end fn.main
