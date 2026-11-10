// trace: D160
@config(default);
import std;

fn.main [] -> void:
    let $a: f32.stable = ~0.1;
    let $b: f32.stable = ~0.1;
    if $a == $b:
        std::io::fmt_print("the same rounding{newln}");
    end if
    return void;
end fn.main
