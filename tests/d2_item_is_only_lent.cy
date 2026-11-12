// trace: D2
import std;

fn eat [$s: Given str[len.max: 32 bytes]] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $xs: std::coll::array<str[len.max: 32 bytes]>[len: 2 items] = ["a", "b"];
    for $s in Lend $xs:
        eat(Give $s);
    end for
    return void;
end fn.main
