// trace: D176, D171

struct reading:
    celsius: i32;
end struct reading

union.tagged lookup <`text`>:
    found[item: Lend from `text` reading];
    missing;
end union.tagged lookup

fn.main [none] returns void:
    let mut $a: reading set_to reading[celsius set_to 4];
    let $l: lookup<`text`: $a> set_to lookup.found[item set_to Lend $a];
    change $a.celsius set_to 1;
    match $l:
        case lookup.found $f:
            return void;
        case lookup.missing:
            return void;
    end match
    return void;
end fn.main
