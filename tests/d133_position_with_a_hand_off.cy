// trace: D133
import std;

fn eat [$s: Given str] returns i32:
    return 1;
end fn eat

fn.main [none] returns void:
    let $s: str set_to "a";
    let $a: std::array<i32>[len: 1 items] set_to [1];
    let $n: i32 set_to $a[pos: eat(Give $s)];
    return void;
end fn.main
