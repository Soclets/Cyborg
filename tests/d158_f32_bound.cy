// trace: D158, D59
import std;
@config(default);

fn.main [] -> void:
    let $x: f32 = ~0.1;
    if $x: f32[min: ~0.1, max: ~0.1]:
        std::io::fmt_print("the same rounding{newln}");
    end if
    return void;
end fn.main
