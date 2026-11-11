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

fn eat [$p: Given pair] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $p: pair set_to pair[a set_to inner[n set_to 1], b set_to inner[n set_to 2]];
    let $l: Lend from $p inner set_to Lend $p.a;
    eat(Give $p);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $l.n);
    return void;
end fn.main
