// trace: D185, D2

struct box:
    s: str;
end struct box

fn eat [$s: Given str] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $b: box set_to box[s set_to "a"];
    eat(Give $b.s);
    return void;
end fn.main
