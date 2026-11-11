// trace: D196, D197
import std;

fn.main [none] returns void:
    let $x: f64.stable set_to 0.5;
    let $y: f64.stable set_to std::sin($x);
    return void;
end fn.main
