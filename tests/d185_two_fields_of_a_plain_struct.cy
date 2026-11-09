// A struct with no duty just goes away once its fields are out.
// trace: D185
import std;
@config(default);

struct pair:
    a: str;
    b: str;
end struct pair

fn eat [$s: Given str] -> void:
    std::println(Lend $s);
    return void;
end fn eat

fn.main [] -> void:
    let $p: pair = pair[a = "x", b = "y"];
    eat(Give $p.a);
    eat(Give $p.b);
    return void;
end fn.main
