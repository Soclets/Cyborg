// Giving away one field of a struct isn't in the standard yet.
// trace: D2
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
