// trace: D114
@config(default);
import std;

fn pair [$a: i32, $b: i32] -> i32:
    labels.any_order [$a as first, $b as second];

    return $a x 10 + $b;
end fn pair

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", pair(second: 1, first: 2));
    return void;
end fn.main
