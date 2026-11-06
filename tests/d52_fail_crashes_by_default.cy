// trace: D52, D96
import std;

fn half [$n: i32] -> i32:
    fails [odd];

    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn.main [] -> void:
    std::println(half(4) as str);
    std::println(half(3) as str);
    std::println("not reached");
    return void;
end fn.main
