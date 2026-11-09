// trace: D196, D197
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = 0.5;
    let $y: f64 = std::sin($x);
    return void;
end fn.main
