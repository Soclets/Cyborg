// trace: D44, D47
import std;
import os;

fn inner [$n: i32] returns i32 fails [odd]:
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n;
end fn inner

fn outer [$n: i32] returns i32 fails [odd]:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $m: i32 set_to inner($n)?;
    std::io::fmt_print(Lend.mut $out, "got {}{newln}", $m);
    return $m;
end fn outer

fn.main [none] returns void:
    let $a: i32 set_to outer(2);
    let $b: i32 set_to outer(3);
    return void;
end fn.main
