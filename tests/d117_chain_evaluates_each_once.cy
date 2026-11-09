// trace: D117, D124
@config(default);
import std;

fn say [$n: i32] -> i32:
    std::io::fmt_print("evaluated {}{newln}", $n);
    return $n;
end fn say

fn.main [] -> void:
    if say(1) < say(2) < say(3):
        std::io::fmt_print("in order{newln}");
    end if
    return void;
end fn.main
