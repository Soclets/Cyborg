// `-0.0` and `0.0` give the same answer here, so nothing hangs on how they compare.
// trace: D158
import std;

fn.main [none] -> void:
    let $z: f64.stable = -0.0;
    let $t: f64.stable[min: -1.0, max: 1.0] = $z;
    std::io::fmt_print("inside{newln}");
    return void;
end fn.main
