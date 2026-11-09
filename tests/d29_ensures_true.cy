// trace: D29, D84
import std;
@config(default);

fn right [$n: i32] -> i32:
    ensures result at_least $n;

    return $n + 1;
end fn right

fn.main [] -> void:
    std::println(right(5) as str);
    return void;
end fn.main
