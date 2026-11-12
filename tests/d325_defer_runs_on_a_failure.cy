// A `defer` runs when a failure leaves its block too.
// trace: D325
import std;
import os;

fn half [$out: Lent.mut os::stdout.unbuffered, $n: i32] -> i32 fails [odd]:
    defer std::io::fmt_print(Lend.mut $out, "leaving half({$n}){newln}");
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = half(Lend.mut $out, 4) ?? -1;
    let $b: i32 = half(Lend.mut $out, 3) ?? -1;
    std::io::fmt_print(Lend.mut $out, "{$a} {$b}{newln}");
    return void;
end fn.main
