// trace: D174
import std;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $m: Lend.mut from $r reading = Lend.mut $r;
    let $n: Lend from $r reading = Lend $m;
    std::println($m.celsius as str);
    std::println($n.celsius as str);
    return void;
end fn.main
