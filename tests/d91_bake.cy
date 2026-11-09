// trace: D91
import std;
@config(default);

fn.pure.baked seconds_in [$days: i32] -> i32:
    return $days x 86_400;
end fn.pure.baked seconds_in

const $week: i32 = seconds_in(7);

fn.main [] -> void:
    let $day: i32 = bake seconds_in(1);
    let $d: i32 = 2;
    let $n: i32 = seconds_in($d);
    std::println($week as str);
    std::println($day as str);
    std::println($n as str);
    return void;
end fn.main
