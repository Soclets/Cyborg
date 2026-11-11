// trace: D122
import std;
import os;

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i64 = 9000000000;
    let $b: u8 = 200;
    let $c: i32[unit: time.seconds] = 3 time.seconds;
    let $d: i32[min: 0, max: 9] = 4;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $c);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $d);
    return void;
end fn.main
