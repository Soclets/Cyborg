// trace: D158, D141
import std;

type.new ratio = f64[min: 0.0, max: 1.0];

fn.main [] -> void:
    let $r: ratio = 0.5;
    let $x: f64 = $r as f64;
    if $x: f64[min: 0.5, max: 0.5]:
        std::println("crossed");
    end if
    return void;
end fn.main
