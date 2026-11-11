// trace: D176, D171

struct reading:
    celsius: i32;
end struct reading

union.tagged lookup <`text`>:
    found[item: Lend from `text` reading];
    missing;
end union.tagged lookup

fn.main [none] returns void:
    let mut $a: reading = reading[celsius = 4];
    let $l: lookup<`text`: $a> = lookup.found[item = Lend $a];
    change $a.celsius = 1;
    match $l:
        case lookup.found $f:
            return void;
        case lookup.missing:
            return void;
    end match
    return void;
end fn.main
