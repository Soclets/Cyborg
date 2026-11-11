// trace: D176
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

union.tagged lookup <`text`>:
    found[item: Lend from `text` reading];
    missing;
end union.tagged lookup

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: reading = reading[celsius = 4];
    let $l: lookup<`text`: $a> = lookup.found[item = Lend $a];
    let $m: lookup<`text`: $a> = lookup.missing;
    match $l:
        case lookup.found $f:
            std::io::fmt_print(Lend.mut $out, "{}{newln}", $f.item.celsius);
        case lookup.missing:
            std::io::fmt_print(Lend.mut $out, "missing{newln}");
    end match
    return void;
end fn.main
