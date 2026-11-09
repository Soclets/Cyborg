// trace: D173
@config(default);
import std;

struct inner:
    n: i32;
end struct inner

struct pair:
    a: inner;
    b: inner;
end struct pair

fn adjust [$x: Lent.mut inner, $y: Lent.mut inner] -> void:
    change $x.n = 10;
    change $y.n = 20;
    return void;
end fn adjust

fn.main [] -> void:
    let mut $p: pair = pair[a = inner[n = 1], b = inner[n = 2]];
    adjust(Lend.mut $p.a, Lend.mut $p.b);
    std::io::fmt_print("{} {}{newln}", $p.a.n, $p.b.n);
    return void;
end fn.main
