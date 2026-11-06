// The item's place is worked out first, so nothing says what this does if the value moves it.
// trace: D133, D2
import std;

fn grow [$xs: Lent.mut std::list<i32>[len.max: inf items]] -> i32:
    return 1;
end fn grow

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = [1];
    change $xs[pos: 1] = grow(Lend.mut $xs);
    return void;
end fn.main
