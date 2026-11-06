// trace: D124
import std;

fn say [$n: i32] -> i32:
    std::println("evaluated ", $n as str);
    return $n;
end fn say

fn.main [] -> void:
    if say(2) < say(1) < say(3):
        std::println("not reached");
    end if
    return void;
end fn.main
