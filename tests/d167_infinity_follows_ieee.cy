// trace: D167, D166
import std;
@config(default);

fn.main [] -> void:
    let $zero: f64.ieee = 0.0;
    let $inf: f64.ieee = 1.0 div $zero;
    let $down: f64.ieee = -1.0 div $zero;
    std::println(($inf + 1.0) as str);
    std::println(($inf x -2.0) as str);
    std::println(($down - $inf) as str);
    std::println(($inf + $down) as str);
    std::println((1.0 div $inf) as str);
    if $down < $inf:
        std::println("ordered");
    end if
    return void;
end fn.main
