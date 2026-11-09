// trace: D164, D159
import std;
@config(default);

fn.main [] -> void:
    let $zero: f64.ieee = 0.0;
    let $inf: f64.ieee = 1.0 div $zero;
    let $nan: f64.ieee = $zero div $zero;
    std::println("still running");
    let $clean: f64 = $nan as f64;
    return void;
end fn.main
