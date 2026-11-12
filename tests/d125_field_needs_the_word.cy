// `change` doesn't stand in for an ownership word.
// trace: D125, D3

struct box:
    s: str[len.max: 32 bytes];
end struct box

fn.main [none] -> void:
    let mut $b: box = box[s = "a"];
    let $s: str[len.max: 32 bytes] = "b";
    change $b.s = $s;
    return void;
end fn.main
