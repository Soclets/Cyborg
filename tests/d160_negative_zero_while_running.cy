// trace: D160, D158
import std;
@config(default);

fn.main [] -> void:
    let $z: f64 = -0.0;
    let $t: f64[min: 0.0, max: 1.0] = $z;
    std::println("fits");
    return void;
end fn.main
