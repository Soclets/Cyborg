// trace: D180, D128
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let mut $readings: std::list<reading>[len.max: inf items] = [reading[celsius = 20]];
    for $r in $readings:
        change $r.celsius = 0;
    end for
    return void;
end fn.main
