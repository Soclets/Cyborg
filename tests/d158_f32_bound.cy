// trace: D158, D59
import std;

fn.main [] -> void:
    let $x: f32 = ~0.1;
    if $x: f32[min: ~0.1, max: ~0.1]:
        std::println("the same rounding");
    end if
    return void;
end fn.main
