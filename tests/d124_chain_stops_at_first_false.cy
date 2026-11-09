// trace: D124
import std;
@config(default);

fn say [$n: i32] -> i32:
    std::io::fmt_print("evaluated {}{newln}", $n);
    return $n;
end fn say

fn.main [] -> void:
    if say(2) < say(1) < say(3):
        std::io::fmt_print("not reached{newln}");
    end if
    return void;
end fn.main
