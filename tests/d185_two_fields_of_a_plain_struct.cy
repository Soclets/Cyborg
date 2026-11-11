// A struct with no duty just goes away once its fields are out.
// trace: D185
import std;
import os;

struct pair:
    a: str;
    b: str;
end struct pair

fn eat [$s: Given str] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn eat

fn.main [none] returns void:
    let $p: pair set_to pair[a set_to "x", b set_to "y"];
    eat(Give $p.a);
    eat(Give $p.b);
    return void;
end fn.main
