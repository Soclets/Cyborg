// trace: D130
import std;
@config(default);

struct a:
    n: i32;
end struct a

struct b:
    n: i32;
end struct b

fn a.%show [$v: a] -> void:
    std::println("a ", $v.n as str);
    return void;
end fn a.%show

fn b.%show [$v: b] -> void:
    std::println("b ", $v.n as str);
    return void;
end fn b.%show

fn show [$n: i32] -> void:
    std::println("plain ", $n as str);
    return void;
end fn show

fn.main [] -> void:
    let $x: a = a[n = 1];
    let $y: b = b[n = 2];
    $x.%show();
    $y.%show();
    show(3);
    return void;
end fn.main
