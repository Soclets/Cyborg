// trace: D42
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to 2147483647;
    let $b: i32 set_to $a plus.wrap 1;
    let $c: u8 set_to 255;
    let $d: u8 set_to $c plus.wrap 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $d);
    return void;
end fn.main
