// trace: D175, D174
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn show [$v: reading] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $v.celsius);
    return void;
end fn show

fn.main [none] returns void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $view: Lend from $r reading = Lend $r;
    show(Lend $view);
    return void;
end fn.main
