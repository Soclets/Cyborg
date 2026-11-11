// trace: D3
import std;

fn.main [none] returns void:
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    change $xs set_to $xs.append(val: 4);
    return void;
end fn.main
