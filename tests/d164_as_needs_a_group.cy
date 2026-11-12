// trace: D164, D116
import std;

fn.main [none] -> void:
    let $a: f64.stable = 1.5;
    let $s: str[len.max: 20 bytes] = $a div 3.0 as str[len.max: 20 bytes];
    return void;
end fn.main
