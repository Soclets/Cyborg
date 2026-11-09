// trace: D161
import std;
@config(default);

fn.main [] -> void:
    let $x: f64 = 0.5;
    std::println.stored($x, $x);
    return void;
end fn.main
