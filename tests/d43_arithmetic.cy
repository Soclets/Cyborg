// trace: D43, D116
import std;

fn.main [] -> void:
    let $a: i32 = 2 + 3 x 4;
    let $b: i32 = (2 + 3) x 4;
    let $c: i32 = 10 - 3 - 2;
    std::println($a as str);
    std::println($b as str);
    std::println($c as str);
    return void;
end fn.main
