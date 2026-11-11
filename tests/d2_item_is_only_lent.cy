// trace: D2
import std;

fn eat [$s: Given str] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $xs: std::array<str>[len: 2 items] = ["a", "b"];
    for $s in $xs:
        eat(Give $s);
    end for
    return void;
end fn.main
