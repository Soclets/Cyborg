// trace: D158
import std;

fn.main [] -> void:
    let mut $x: f64 = 2.0;
    for $i in range[from: 1, to: 9]:
        change $x = $x x $x;
    end for
    std::println("2^512 is fine");
    return void;
end fn.main
