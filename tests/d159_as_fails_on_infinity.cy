// trace: D159
import std;
@config(default);

fn.main [] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    std::println("before");
    let $clean: f64 = $x as f64;
    std::println("after");
    return void;
end fn.main
