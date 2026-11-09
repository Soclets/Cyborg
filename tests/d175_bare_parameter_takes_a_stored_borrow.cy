// trace: D175, D174
import std;
@config(default);

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn show [$v: reading] -> void:
    std::println($v.celsius as str);
    return void;
end fn show

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $view: Lend from $r reading = Lend $r;
    show(Lend $view);
    return void;
end fn.main
