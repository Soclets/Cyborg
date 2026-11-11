// trace: D180, D2
import std;

fn change_all [$xs: std::array<i32>[len: 2 items]] returns void:
    for $n in Lend.mut $xs:
        change $n set_to 0;
    end for
    return void;
end fn change_all

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 2 items] set_to [1, 2];
    change_all(Lend $xs);
    return void;
end fn.main
