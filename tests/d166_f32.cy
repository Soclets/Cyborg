// trace: D166, D159
import std;

fn.main [] -> void:
    let $zero: f32.ieee = 0.0;
    let $inf: f32.ieee = 1.0 div $zero;
    std::println($inf as str);
    std::println(($zero div $zero) as str);
    return void;
end fn.main
