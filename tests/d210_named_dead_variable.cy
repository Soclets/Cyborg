// A named hole reads its variable, so a given-away one can't print.
// trace: D210, D2
import std;
import os;

fn eat [$s: Given str[len.max: 32 bytes]] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: str[len.max: 32 bytes] = "a";
    eat(Give $s);
    std::io::fmt_print(Lend.mut $out, "{$s}{newln}");
    return void;
end fn.main
