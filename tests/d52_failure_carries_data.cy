// How an arm reads a failure's data isn't decided, so this only shows it raised.
// trace: D52, D47
import std;
import os;

fn half [$n: i32] returns i32 fails [negative[value: i32]]:
    if $n below 0:
        fail negative[value set_to $n];
    end if
    return $n;
end fn half

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $n: i32 set_to half(-3);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    return void;
end fn.main
