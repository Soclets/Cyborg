// `change` doesn't stand in for an ownership word.
// trace: D125, D3

struct box:
    s: str;
end struct box

fn.main [none] returns void:
    let mut $b: box set_to box[s set_to "a"];
    let $s: str set_to "b";
    change $b.s set_to $s;
    return void;
end fn.main
