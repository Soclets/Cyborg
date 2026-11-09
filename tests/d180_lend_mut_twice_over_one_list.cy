// trace: D180, D2
import std;
@config(default);

fn.main [] -> void:
    let mut $xs: std::array<i32>[len: 2 items] = [1, 2];
    for $a in Lend.mut $xs:
        for $b in Lend.mut $xs:
            change $b = $a;
        end for
    end for
    return void;
end fn.main
