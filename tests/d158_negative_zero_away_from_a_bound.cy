// `-0.0` and `0.0` give the same answer here, so nothing hangs on how they compare.
// trace: D158
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $z: f64.stable set_to -0.0;
    let $t: f64.stable[min: -1.0, max: 1.0] set_to $z;
    std::io::fmt_print(Lend.mut $out, "inside{newln}");
    return void;
end fn.main
