// trace: D162, D161
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = ~0.1;
    std::println($x as str);
    std::println.stored($x);
    return void;
end fn.main
