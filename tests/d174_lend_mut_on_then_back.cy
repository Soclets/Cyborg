// `$n` isn't used after its change, so `$m` is free again.
// trace: D174, D169
import std;
@config(default);

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $m: Lend.mut from $r reading = Lend.mut $r;
    let $n: Lend.mut from $r reading = Lend.mut $m;
    change $n.celsius = 2;
    change $m.time = 7;
    std::println($r.celsius as str, " ", $r.time as str);
    return void;
end fn.main
