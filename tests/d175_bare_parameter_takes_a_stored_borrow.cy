// trace: D175, D174
@config(default);
import std;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn show [$v: reading] -> void:
    std::io::fmt_print("{}{newln}", $v.celsius);
    return void;
end fn show

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $view: Lend from $r reading = Lend $r;
    show(Lend $view);
    return void;
end fn.main
