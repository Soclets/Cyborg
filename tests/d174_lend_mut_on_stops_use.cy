// trace: D174
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    let $m: Lend.mut from $r reading set_to Lend.mut $r;
    let $n: Lend.mut from $r reading set_to Lend.mut $m;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $m.celsius);
    change $n.celsius set_to 2;
    return void;
end fn.main
