// trace: D174
@config(default);
import std;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $m: Lend.mut from $r reading = Lend.mut $r;
    let $n: Lend from $r reading = Lend $m;
    change $m.celsius = 1;
    std::io::fmt_print("{}{newln}", $n.celsius);
    return void;
end fn.main
