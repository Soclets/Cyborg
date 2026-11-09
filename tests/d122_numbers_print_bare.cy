// trace: D122
import std;
@config(default);

fam.unit time:
    time.seconds: base;
end fam.unit time

fn.main [] -> void:
    let $a: i64 = 9000000000;
    let $b: u8 = 200;
    let $c: i32.time.seconds = 3 time.seconds;
    let $d: i32[min: 0, max: 9] = 4;
    std::io::fmt_print("{}{newln}", $a);
    std::io::fmt_print("{}{newln}", $b);
    std::io::fmt_print("{}{newln}", $c);
    std::io::fmt_print("{}{newln}", $d);
    return void;
end fn.main
