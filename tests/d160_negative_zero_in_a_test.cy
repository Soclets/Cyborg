// trace: D160, D158
import std;

fn.main [none] -> void:
    let $z: f64.stable = -0.0;
    if $z: f64.stable[min: 0.0, max: 1.0]:
        std::io::fmt_print("inside{newln}");
    end if
    return void;
end fn.main
