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
    let $n: Lend from $r reading set_to Lend $m;
    change $m.celsius set_to 1;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n.celsius);
    return void;
end fn.main
