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
    let $n: Lend.mut from $r reading = Lend.mut $m;
    std::io::fmt_print("{}{newln}", $m.celsius);
    change $n.celsius = 2;
    return void;
end fn.main
