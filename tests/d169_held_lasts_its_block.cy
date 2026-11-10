// trace: D169
import std;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.held from $r reading = Lend $r;
    std::io::fmt_print("{}{newln}", $lock.celsius);
    change $r.celsius = 22;
    return void;
end fn.main
