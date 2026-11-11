// trace: D180, D2
import std;

fn change_all [$xs: std::array<i32>[len: 2 items]] -> void:
    for $n in Lend.mut $xs:
        change $n = 0;
    end for
    return void;
end fn change_all

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    change_all(Lend $xs);
    return void;
end fn.main
