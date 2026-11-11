// trace: D178, D115
import std;

fn.main [none] returns void:
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.%append(val: 4);
    return void;
end fn.main
