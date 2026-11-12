// Giving away one item would leave a hole in the collection, and isn't decided.
// trace: D133, D2
import std;

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $a: std::coll::array<str>[len: 1 items] = ["hi"];
    eat(Give $a[pos: 1]);
    return void;
end fn.main
