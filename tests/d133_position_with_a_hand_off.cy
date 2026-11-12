// trace: D133
import std;

fn eat [$s: Given str[len.max: 32 bytes]] -> i32:
    return 1;
end fn eat

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    let $a: std::coll::array<i32>[len: 1 items] = [1];
    let $n: i32 = $a[pos: eat(Give $s)];
    return void;
end fn.main
