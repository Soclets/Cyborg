// trace: D114
import std;

fn pair [$a: i32, $b: i32] -> i32:
    labels.any_order [$a as first, $b as second];

    return $a x 10 + $b;
end fn pair

fn.main [] -> void:
    std::println(pair(second: 1, first: 2) as str);
    return void;
end fn.main
