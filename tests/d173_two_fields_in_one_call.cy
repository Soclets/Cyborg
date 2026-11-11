// trace: D173
import std;
import os;

struct inner:
    n: i32;
end struct inner

struct pair:
    a: inner;
    b: inner;
end struct pair

fn adjust [$x: Lent.mut inner, $y: Lent.mut inner] returns void:
    change $x.n set_to 10;
    change $y.n set_to 20;
    return void;
end fn adjust

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $p: pair set_to pair[a set_to inner[n set_to 1], b set_to inner[n set_to 2]];
    adjust(Lend.mut $p.a, Lend.mut $p.b);
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $p.a.n, $p.b.n);
    return void;
end fn.main
