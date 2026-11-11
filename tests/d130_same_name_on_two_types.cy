// trace: D130
import std;
import os;

struct a:
    n: i32;
end struct a

struct b:
    n: i32;
end struct b

fn a.%show [$v: a] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "a {}{newln}", $v.n);
    return void;
end fn a.%show

fn b.%show [$v: b] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "b {}{newln}", $v.n);
    return void;
end fn b.%show

fn show [$n: i32] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "plain {}{newln}", $n);
    return void;
end fn show

fn.main [none] returns void:
    let $x: a = a[n = 1];
    let $y: b = b[n = 2];
    $x.%show();
    $y.%show();
    show(3);
    return void;
end fn.main
