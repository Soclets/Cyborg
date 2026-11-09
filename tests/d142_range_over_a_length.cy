// trace: D142, D128
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [4, 5, 6];
    for $i in range[from: 1, to: $xs.len]:
        std::println($i as str, " holds ", $xs[pos: $i] as str);
    end for
    for $i in range[from: 1 items, before: $xs.len]:
        std::println($i as str);
    end for
    for $i in range[from: 1 items, to: 2 items]:
        let $kept: i32.items = $i;
        std::println($kept as str);
    end for
    return void;
end fn.main
