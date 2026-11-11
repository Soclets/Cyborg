// trace: D180, D128
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let mut $readings: std::list<reading>[len.max: inf items] set_to [reading[celsius set_to 20]];
    for $r in $readings:
        change $r.celsius set_to 0;
    end for
    return void;
end fn.main
