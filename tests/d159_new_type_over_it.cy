// trace: D159, D141
import std;

type.new sensor = f64.ieee;

fn.main [none] -> void:
    let $r: sensor = 0.5;
    let $i: f64.ieee = $r as f64.ieee;
    let $clean: f64.stable = $i as f64.stable;
    std::io::fmt_print("ok{newln}");
    return void;
end fn.main
