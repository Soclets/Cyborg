// Left to right as written, which is the guess in the queue.
// trace: D133
import std;

fn say [$n: i32] -> i32:
    std::io::fmt_print("say {}{newln}", $n);
    return $n;
end fn say

fn.main [none] -> void:
    let mut $a: std::array<i32>[len: 1 items] = [1];
    change $a[pos: say(1)] = say(2);
    return void;
end fn.main
