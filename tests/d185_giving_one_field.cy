// trace: D185, D2
@config(default);

struct box:
    s: str;
end struct box

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $b: box = box[s = "a"];
    eat(Give $b.s);
    return void;
end fn.main
