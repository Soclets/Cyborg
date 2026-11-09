// trace: D173
import std;
@config(default);

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

fn.main [] -> void:
    let $p: pair = pair[a = inner[n = 1], b = inner[n = 2]];
    let $l: Lend from $p inner = Lend $p.a;
    eat(Give $p);
    std::println($l.n as str);
    return void;
end fn.main
