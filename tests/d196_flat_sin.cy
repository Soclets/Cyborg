// trace: D196, D197
import std;

fn.main [none] -> void:
    let $x: f64.stable = 0.5;
    let $y: f64.stable = std::sin($x);
    return void;
end fn.main
