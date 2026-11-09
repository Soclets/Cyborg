// trace: D164, D83
import std;
@config(default);

const $half: f64 = 1.0 div 2.0;

fn.main [] -> void:
    std::println($half as str);
    return void;
end fn.main
