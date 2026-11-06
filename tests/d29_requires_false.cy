// trace: D29, D84
import std;

fn half [$n: i32] -> i32:
    requires $n at_least 0;

    return $n div.trunc 2;
end fn half

fn.main [] -> void:
    std::println(half(4) as str);
    std::println(half(-4) as str);
    return void;
end fn.main
