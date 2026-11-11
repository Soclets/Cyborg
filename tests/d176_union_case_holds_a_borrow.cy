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

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: reading set_to reading[celsius set_to 4];
    let $l: lookup<`text`: $a> set_to lookup.found[item set_to Lend $a];
    let $m: lookup<`text`: $a> set_to lookup.missing;
    match $l:
        case lookup.found $f:
            std::io::fmt_print(Lend.mut $out, "{}{newln}", $f.item.celsius);
        case lookup.missing:
            std::io::fmt_print(Lend.mut $out, "missing{newln}");
    end match
    return void;
end fn.main
