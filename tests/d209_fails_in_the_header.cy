// trace: D209, D52
import std;
import os;

fn half [$n: i32] -> i32 fails [odd, negative[val: i32]]:
    requires $n at_least -100;
    if $n < 0:
        fail negative[val = $n];
    end if
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $h: i32 = half(8) ?? 0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $h);
    return void;
end fn.main
