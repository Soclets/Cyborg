// D140 brings these two in, and they work like the other integers.
// trace: D140, D42
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: u32 set_to 4294967295;
    let $b: u64 set_to 18446744073709551615;
    let $c: u32 set_to $a plus.wrap 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    return void;
end fn.main
