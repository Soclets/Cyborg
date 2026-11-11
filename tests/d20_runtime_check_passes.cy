// trace: D20
import std;
import os;

fn use_score [$s: i32[min: 0, max: 100]] -> i32:
    return $s + 1;
end fn use_score

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $n: i32 = 100;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", use_score($n));
    return void;
end fn.main
