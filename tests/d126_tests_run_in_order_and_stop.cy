// trace: D126
@config(default);
import std;

fn say [$n: i32] -> bool:
    std::io::fmt_print("test {}{newln}", $n);
    return $n == 2;
end fn say

fn.main [] -> void:
    if say(1):
        std::io::fmt_print("one{newln}");
    else if say(2):
        std::io::fmt_print("two{newln}");
    else if say(3):
        std::io::fmt_print("three{newln}");
    else:
        std::io::fmt_print("none{newln}");
    end if
    return void;
end fn.main
