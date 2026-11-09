// Knowing the range proves nothing here, since it goes past the array, so the check stays.
// trace: D147
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 2 items] = [4, 5];
    for $i in range[from: 1 items, to: 3 items]:
        std::println($xs[pos: $i] as str);
    end for
    return void;
end fn.main
