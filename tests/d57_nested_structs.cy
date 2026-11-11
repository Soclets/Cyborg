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

fn read [$i: inner] returns i32:
    return $i.n;
end fn read

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $first: inner set_to inner[n set_to 1];
    let $o: outer set_to outer[a set_to Give $first, b set_to inner[n set_to 2]];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $o.a.n);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", read(Lend $o.b));
    return void;
end fn.main
