// trace: D2
import std;

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $xs: std::coll::array<str>[len: 2 items] = ["a", "b"];
    for $s in $xs:
        eat(Give $s);
    end for
    return void;
end fn.main
