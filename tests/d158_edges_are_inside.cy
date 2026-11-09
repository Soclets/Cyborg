// trace: D158
import std;
@config(default);

fn.main [] -> void:
    let $low: f64[min: -40.0, max: 125.0] = -40.0;
    let $high: f64[min: -40.0, max: 125.0] = 125.0;
    std::println("both");
    return void;
end fn.main
