// trace: D121, D86, D91
import std;
@config(default);

fn.pure area [$w: i32, $h: i32] -> i32:
    return $w x $h;
end fn.pure area

fn.pure.baked seconds_in [$days: i32] -> i32:
    return $days x 86_400;
end fn.pure.baked seconds_in

fn.main [] -> void:
    std::println(area(3, 4) as str);
    std::println(seconds_in(2) as str);
    return void;
end fn.main
