// Arithmetic gives back a plain `i32`, and `change` checks it.
// trace: D20
import std;

fn.main [] -> void:
    let mut $b: i32[min: 0, max: 100] = 100;
    change $b = $b + 1;
    std::println("after");
    return void;
end fn.main
