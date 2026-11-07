// trace: D164, D158
import std;

fn.main [] -> void:
    let $zero: f64 = 0.0;
    std::println("before");
    let $a: f64 = 1.0 div $zero;
    std::println("after");
    return void;
end fn.main
