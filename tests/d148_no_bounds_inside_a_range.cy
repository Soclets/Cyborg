// trace: D148, D133
import std;

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    for $i in range[from: 1, to: $xs.len[min: 100]]:
        return void;
    end for
    return void;
end fn.main
