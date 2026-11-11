// trace: D209, D52
import std;
import os;

fn half [$n: i32] returns i32 fails [odd, negative[val: i32]]:
    requires $n at_least -100;
    if $n below 0:
        fail negative[val set_to $n];
    end if
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $h: i32 set_to half(8) ?? 0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $h);
    return void;
end fn.main
