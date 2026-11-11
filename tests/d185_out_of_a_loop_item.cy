// trace: D185, D180
import std;

struct box:
    s: str;
end struct box

fn eat [$s: Given str] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $xs: std::list<box>[len.max: inf items] set_to [box[s set_to "a"]];
    for $b in $xs:
        eat(Give $b.s);
    end for
    return void;
end fn.main
