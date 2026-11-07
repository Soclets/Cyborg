// trace: D159, D141
import std;

type.new sensor = f64.ieee;

fn.main [] -> void:
    let $r: sensor = 0.5;
    let $i: f64.ieee = $r as f64.ieee;
    let $clean: f64 = $i as f64;
    std::println("ok");
    return void;
end fn.main
