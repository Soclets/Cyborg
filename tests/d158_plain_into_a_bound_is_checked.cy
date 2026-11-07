// trace: D158, D20
import std;

fn.main [] -> void:
    let $x: f64 = 130.5;
    std::println("before");
    let $t: f64[min: -40.0, max: 125.0] = $x;
    std::println("after");
    return void;
end fn.main
