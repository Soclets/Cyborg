// trace: D166, D159
import std;
@config(default);

fn.main [] -> void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    let $nan: f64.ieee = $x - $x;
    std::println($nan as str);
    std::println((-$nan) as str);
    std::println.stored($nan);
    return void;
end fn.main
