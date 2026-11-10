// trace: D173
import std;

struct inner:
    n: i32;
end struct inner

struct pair:
    a: inner;
    b: inner;
end struct pair

fn.main [none] -> void:
    let mut $p: pair = pair[a = inner[n = 1], b = inner[n = 2]];
    let $l: Lend from $p inner = Lend $p.a;
    change $p.a.n = 3;
    std::io::fmt_print("{}{newln}", $l.n);
    return void;
end fn.main
