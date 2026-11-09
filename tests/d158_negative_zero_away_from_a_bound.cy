// `-0.0` and `0.0` give the same answer here, so nothing hangs on how they compare.
// trace: D158
import std;
@config(default);

fn.main [] -> void:
    let $z: f64 = -0.0;
    let $t: f64[min: -1.0, max: 1.0] = $z;
    std::println("inside");
    return void;
end fn.main
