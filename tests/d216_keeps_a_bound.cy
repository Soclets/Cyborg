// Exactly the type of the right side, bound and all.
// trace: D216, D20
import std;

fn.main [none] -> void:
    let $x: i32[min: 0, max: 9] = 3;
    let $y := $x;
    let $z: i32[min: 0, max: 9] = $y;
    std::io::fmt_print("{}{newln}", $z);
    return void;
end fn.main
