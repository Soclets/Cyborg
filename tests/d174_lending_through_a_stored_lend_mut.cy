// trace: D174, D2
import std;

struct reading:
    celsius: i32;
end struct reading

fn replace [$r: Lent.mut reading] -> void:
    change $r = reading[celsius = 0];
    return void;
end fn replace

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21];
    let $b: Lend.mut from $r reading = Lend.mut $r;
    replace(Lend.mut $b);
    change $b.celsius = $b.celsius + 30;
    std::println($r.celsius as str);
    return void;
end fn.main
