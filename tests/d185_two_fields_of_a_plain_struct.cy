// A struct with no duty just goes away once its fields are out.
// trace: D185
import std;
import os;

struct pair:
    a: str[len.max: 32 bytes];
    b: str[len.max: 32 bytes];
end struct pair

fn eat [$s: Given str[len.max: 32 bytes]] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn eat

fn.main [none] -> void:
    let $p: pair = pair[a = "x", b = "y"];
    eat(Give $p.a);
    eat(Give $p.b);
    return void;
end fn.main
