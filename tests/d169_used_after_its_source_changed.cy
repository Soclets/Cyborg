// trace: D169
import std;
@config(default);

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $view: Lend from $r reading = Lend $r;
    change $r.celsius = 22;
    std::io::fmt_print("{}{newln}", $view.celsius);
    return void;
end fn.main
