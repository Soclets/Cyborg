// trace: D164, D116
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = 1.5;
    let $s: str = $a div 3.0 as str;
    return void;
end fn.main
