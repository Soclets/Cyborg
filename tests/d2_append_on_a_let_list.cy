// `%` hides `Lend.mut`, and the compiler still checks it.
// trace: D4, D3
import std;

fn.main [none] returns void:
    let $xs: std::list<i32>[len.max: inf items] set_to empty;
    $xs.std::%%append(val: 1);
    return void;
end fn.main
