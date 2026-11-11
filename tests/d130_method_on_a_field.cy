// trace: D130, D125
import std;
import os;

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
end struct outer

fn inner.%%bump [$i: Lent.mut inner] returns void:
    change $i.n = $i.n + 1;
    return void;
end fn inner.%%bump

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $o: outer = outer[a = inner[n = 1]];
    $o.a.%%bump();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $o.a.n);
    return void;
end fn.main
