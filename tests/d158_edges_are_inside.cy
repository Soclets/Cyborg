// trace: D158
import std;

fn.main [none] -> void:
    let $low: f64.stable[min: -40.0, max: 125.0] = -40.0;
    let $high: f64.stable[min: -40.0, max: 125.0] = 125.0;
    std::io::fmt_print("both{newln}");
    return void;
end fn.main
