// trace: D185, D180
import std;
import os;

struct box:
    s: str[len.max: 32 bytes];
end struct box

fn eat [$s: Given str[len.max: 32 bytes]] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let $xs: std::coll::list<box>[len.max: inf items] = std::coll::list.from([box[s = "a"]], alloc: Lend $heap);
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    for $b in $xs:
        eat(Give $b.s);
    end for
    return void;
end fn.main
