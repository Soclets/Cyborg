// Applying the first fix makes the later error go, so it points back at it.
// trace: D207, D215
import std;

fn.main [none] -> void:
    let $a: f64.stable = 0.1;
    let $b: f64.stable = $a;
    return void;
end fn.main
