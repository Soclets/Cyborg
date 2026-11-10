// trace: D169
import std;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.mut.held from $r reading = Lend.mut $r;
    change $lock.celsius = 5;
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn.main
