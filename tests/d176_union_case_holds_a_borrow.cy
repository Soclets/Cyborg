// trace: D176
import std;

struct reading:
    celsius: i32;
end struct reading

union.tagged lookup <`text`>:
    found[item: Lend from `text` reading]
    missing
end union.tagged lookup

fn.main [] -> void:
    let $a: reading = reading[celsius = 4];
    let $l: lookup<`text`: $a> = lookup.found[item = Lend $a];
    let $m: lookup<`text`: $a> = lookup.missing;
    match $l:
        case lookup.found $f:
            std::println($f.item.celsius as str);
        case lookup.missing:
            std::println("missing");
    end match
    return void;
end fn.main
