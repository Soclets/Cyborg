// A message names a float type with its kind.
// trace: D257
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable = 0.5;
    let $y: i32 = $x;
    return void;
end fn.main
