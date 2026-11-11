// Left to right as written, which is the guess in the queue.
// trace: D133
import std;
import os;

fn say [$n: i32] returns i32:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "say {}{newln}", $n);
    return $n;
end fn say

fn.main [none] returns void:
    let mut $a: std::array<i32>[len: 1 items] = [1];
    change $a[pos: say(1)] = say(2);
    return void;
end fn.main
