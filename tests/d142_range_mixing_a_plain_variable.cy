// trace: D142, D128
import std;

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    let $n: u64 = 1;
    for $i in range[from: $n, to: $xs.len]:
        return void;
    end for
    return void;
end fn.main
