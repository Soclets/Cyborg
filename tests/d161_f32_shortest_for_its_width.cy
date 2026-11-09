// trace: D161
import std;
@config(default);

fn.main [] -> void:
    let $x: f32 = ~0.1;
    std::println($x as str);
    return void;
end fn.main
