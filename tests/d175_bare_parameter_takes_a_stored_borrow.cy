// trace: D175, D174
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn show [$v: reading] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $v.celsius);
    return void;
end fn show

fn.main [none] returns void:
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    let $view: Lend from $r reading set_to Lend $r;
    show(Lend $view);
    return void;
end fn.main
