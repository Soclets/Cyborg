// A float says which kind it is, so a bare `f64` is refused, asking which.
// trace: D257
@config(default);
import std;

fn.main [] -> void:
    let $x: f64 = 0.5;
    return void;
end fn.main
