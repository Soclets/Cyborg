// The test is how to handle a bad value yourself.
// trace: D20, D9
import std;
import os;

fn use_score [$s: i32[min: 0, max: 100]] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "score {}{newln}", $s);
    return void;
end fn use_score

fn try [$n: i32] returns void:
    if $n: i32[min: 0, max: 100]:
        use_score($n);
    end if
    return void;
end fn try

fn.main [none] returns void:
    try(50);
    try(150);
    return void;
end fn.main
