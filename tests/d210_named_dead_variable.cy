// A named hole reads its variable, so a given-away one can't print.
// trace: D210, D2
@config(default);
import std;

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $s: str = "a";
    eat(Give $s);
    std::io::fmt_print("{$s}{newln}");
    return void;
end fn.main
