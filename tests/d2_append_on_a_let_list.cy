// `%` hides `Lend.mut`, and the compiler still checks it.
// trace: D4, D3
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::list<i32>[len.max: inf items] = empty;
    $xs.std::%append(val: 1);
    return void;
end fn.main
