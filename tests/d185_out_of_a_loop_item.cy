// trace: D185, D180
import std;

struct box:
    s: str;
end struct box

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $xs: std::list<box>[len.max: inf items] = [box[s = "a"]];
    for $b in $xs:
        eat(Give $b.s);
    end for
    return void;
end fn.main
