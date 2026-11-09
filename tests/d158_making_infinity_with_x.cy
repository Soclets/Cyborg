// 2 squared ten times is 2^1024, one past what an `f64` holds.
// trace: D158
import std;
@config(default);

fn.main [] -> void:
    let mut $x: f64 = 2.0;
    for $i in range[from: 1, to: 10]:
        std::println($i as str);
        change $x = $x x $x;
    end for
    return void;
end fn.main
