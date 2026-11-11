// trace: D115
import std;

fn.main [none] returns void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.%append(val: 4);
    return void;
end fn.main
