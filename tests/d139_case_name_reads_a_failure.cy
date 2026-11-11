// trace: D139, D47
import std;
import os;

fn half [$n: i32] returns i32 fails [negative[val: i32], odd]:
    if $n below 0:
        fail negative[val set_to $n];
    end if
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn show [$n: i32] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $h: i32 set_to half($n) otherwise:
        case negative $f:
            std::io::fmt_print(Lend.mut $out, "negative {}{newln}", $f.val);
            instead 0;
        case odd:
            instead -1;
    end otherwise
    std::io::fmt_print(Lend.mut $out, "got {}{newln}", $h);
    return void;
end fn show

fn.main [none] returns void:
    show(4);
    show(-3);
    show(3);
    return void;
end fn.main
