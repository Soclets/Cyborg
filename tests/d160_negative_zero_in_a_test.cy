// trace: D160, D158
@config(default);
import std;

fn.main [] -> void:
    let $z: f64 = -0.0;
    if $z: f64[min: 0.0, max: 1.0]:
        std::io::fmt_print("inside{newln}");
    end if
    return void;
end fn.main
