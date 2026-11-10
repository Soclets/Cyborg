// trace: D130
import std;

struct a:
    n: i32;
end struct a

struct b:
    n: i32;
end struct b

fn a.%show [$v: a] -> void:
    std::io::fmt_print("a {}{newln}", $v.n);
    return void;
end fn a.%show

fn b.%show [$v: b] -> void:
    std::io::fmt_print("b {}{newln}", $v.n);
    return void;
end fn b.%show

fn show [$n: i32] -> void:
    std::io::fmt_print("plain {}{newln}", $n);
    return void;
end fn show

fn.main [none] -> void:
    let $x: a = a[n = 1];
    let $y: b = b[n = 2];
    $x.%show();
    $y.%show();
    show(3);
    return void;
end fn.main
