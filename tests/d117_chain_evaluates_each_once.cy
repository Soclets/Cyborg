// trace: D117, D124
import std;
@config(default);

fn say [$n: i32] -> i32:
    std::println("evaluated ", $n as str);
    return $n;
end fn say

fn.main [] -> void:
    if say(1) < say(2) < say(3):
        std::println("in order");
    end if
    return void;
end fn.main
