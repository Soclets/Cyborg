// Trigonometry asks for an angle in radians, so a plain number is refused.
// trace: D245
import std;

fn.main [none] returns void:
    let $s: f64.stable = std::math::sin(90.0);
    return void;
end fn.main
