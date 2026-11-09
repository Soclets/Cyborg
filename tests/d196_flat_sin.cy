// trace: D196, D197
@config(default);
import std;

fn.main [] -> void:
    let $x: f64 = 0.5;
    let $y: f64 = std::sin($x);
    return void;
end fn.main
