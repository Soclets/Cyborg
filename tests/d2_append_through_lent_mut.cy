// trace: D2, D4
@config(default);
import std;

fn add [$xs: Lent.mut std::list<i32>[len.max: inf items]] -> void:
    $xs.std::%%append(val: 1);
    return void;
end fn add

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = empty;
    add(Lend.mut $xs);
    add(Lend.mut $xs);
    std::io::fmt_print("{}{newln}", $xs.len);
    return void;
end fn.main
