// Trigonometry asks for an angle in radians, so a plain number is refused.
// trace: D245
@config(default);
import std;

fn.main [] -> void:
    let $s: f64 = std::math::sin(90.0);
    return void;
end fn.main
