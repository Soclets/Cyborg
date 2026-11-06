// trace: D126
import std;

fn say [$n: i32] -> bool:
    std::println("test ", $n as str);
    return $n == 2;
end fn say

fn.main [] -> void:
    if say(1):
        std::println("one");
    else if say(2):
        std::println("two");
    else if say(3):
        std::println("three");
    else:
        std::println("none");
    end if
    return void;
end fn.main
