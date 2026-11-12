// trace: D185, D180
import std;
import os;

struct box:
    s: str;
end struct box

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $xs: std::list<box>[len.max: inf items] = [box[s = "a"]];
    defer std::list.done(Give $xs, alloc: Lend $heap);
    for $b in $xs:
        eat(Give $b.s);
    end for
    return void;
end fn.main
