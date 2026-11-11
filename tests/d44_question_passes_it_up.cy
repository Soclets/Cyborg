// trace: D44, D47
import std;
import os;

fn inner [$n: i32] -> i32 fails [odd]:
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n;
end fn inner

fn outer [$n: i32] -> i32 fails [odd]:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $m: i32 = inner($n)?;
    std::io::fmt_print(Lend.mut $out, "got {}{newln}", $m);
    return $m;
end fn outer

fn.main [none] -> void:
    let $a: i32 = outer(2);
    let $b: i32 = outer(3);
    return void;
end fn.main
