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

fn eat [$p: Given pair] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $p: pair = pair[a = inner[n = 1], b = inner[n = 2]];
    let $l: Lend from $p inner = Lend $p.a;
    eat(Give $p);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $l.n);
    return void;
end fn.main
