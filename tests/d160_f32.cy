// trace: D160
import std;
@config(default);

fn.main [] -> void:
    let $a: f32 = ~0.1;
    let $b: f32 = ~0.1;
    if $a == $b:
        std::println("the same rounding");
    end if
    return void;
end fn.main
