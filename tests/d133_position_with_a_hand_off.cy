// trace: D133
import std;
@config(default);

fn eat [$s: Given str] -> i32:
    return 1;
end fn eat

fn.main [] -> void:
    let $s: str = "a";
    let $a: std::array<i32>[len: 1 items] = [1];
    let $n: i32 = $a[pos: eat(Give $s)];
    return void;
end fn.main
