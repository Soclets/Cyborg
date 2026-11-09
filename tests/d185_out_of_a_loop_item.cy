// trace: D185, D180
import std;
@config(default);

struct box:
    s: str;
end struct box

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $xs: std::list<box>[len.max: inf items] = [box[s = "a"]];
    for $b in $xs:
        eat(Give $b.s);
    end for
    return void;
end fn.main
