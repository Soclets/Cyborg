// trace: D44, D47
@config(default);
import std;

fn inner [$n: i32] -> i32:
    fails [odd];

    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n;
end fn inner

fn outer [$n: i32] -> i32:
    fails [odd];

    let $m: i32 = inner($n)?;
    std::io::fmt_print("got {}{newln}", $m);
    return $m;
end fn outer

fn.main [] -> void:
    let $a: i32 = outer(2);
    let $b: i32 = outer(3);
    return void;
end fn.main
