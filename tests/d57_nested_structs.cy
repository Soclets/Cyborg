// trace: D57
import std;
import os;

struct inner:
    n: i32;
end struct inner

struct outer:
    a: inner;
    b: inner;
end struct outer

fn read [$i: inner] -> i32:
    return $i.n;
end fn read

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $first: inner = inner[n = 1];
    let $o: outer = outer[a = Give $first, b = inner[n = 2]];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $o.a.n);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", read(Lend $o.b));
    return void;
end fn.main
