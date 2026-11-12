// `split` is a list method from `std`, so it says its kit, as `simd_chunks` does.
// trace: D115, D382
import std;

fn.main [none] -> void:
    let mut $xs: std::coll::list<i64>[len.max: 8 items] = [1, 2, 3];
    let $h: std::coll::halves.mut<i64> = $xs.%%split.mut(at.items: 1);
    return void;
end fn.main
