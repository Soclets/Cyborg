// trace: D180, D2
import std;

fn.main [] -> void:
    let mut $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        std::println($xs.len as str);
    end for
    return void;
end fn.main
