// trace: D164, D116
import std;

fn.main [none] returns void:
    let $a: f64.stable set_to 1.5;
    let $s: str set_to $a div 3.0 as str;
    return void;
end fn.main
