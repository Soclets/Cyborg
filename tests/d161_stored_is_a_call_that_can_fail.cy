// trace: D161, D95
import std;
@config(default);

fn.pure p [$x: f64] -> f64:
    std::println.stored($x);
    return $x;
end fn.pure p

fn.main [] -> void:
    return void;
end fn.main
