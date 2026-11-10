// trace: D164, D116
@config(default);
import std;

fn.main [] -> void:
    let $a: f64.stable = 1.5;
    let $s: str = $a div 3.0 as str;
    return void;
end fn.main
