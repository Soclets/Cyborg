// trace: D116
import std;

fn.main [] -> void:
    let $x: i32 = 5;
    let $y: i32 = -$x;
    let $z: i32 = -7 + -$y x 2;
    let $f: f64 = 0.5;
    let $g: f64 = -$f;
    std::println($y as str);
    std::println($z as str);
    if $g == -0.5:
        std::println("negated");
    end if
    return void;
end fn.main
