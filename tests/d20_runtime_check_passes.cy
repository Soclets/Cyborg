// trace: D20
import std;
import os;

fn use_score [$s: i32[min: 0, max: 100]] returns i32:
    return $s + 1;
end fn use_score

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i32 set_to 100;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", use_score($n));
    return void;
end fn.main
