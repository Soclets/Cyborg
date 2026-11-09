// trace: D180
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let mut $readings: std::list<reading>[len.max: inf items] = [reading[celsius = 20], reading[celsius = 30]];
    for $r in Lend.mut $readings:
        change $r.celsius = $r.celsius + 1;
    end for
    for $r in $readings:
        std::io::fmt_print("{}{newln}", $r.celsius);
    end for
    return void;
end fn.main
