// trace: D166, D159
import std;

fn.main [] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    std::println($x as str);
    std::println((-$x) as str);
    std::println.stored($x);
    std::println.stored(-$x);
    return void;
end fn.main
