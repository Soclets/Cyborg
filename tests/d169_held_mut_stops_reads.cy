// trace: D169
import std;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.mut.held from $r reading = Lend.mut $r;
    change $lock.celsius = 5;
    std::println($r.celsius as str);
    return void;
end fn.main
