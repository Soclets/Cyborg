// trace: D180, D128
import std;

fn.main [] -> void:
    let mut $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        if $n == 2:
            break;
        end if
        change $n = 0;
    end for
    for $n in $xs:
        std::println($n as str);
    end for
    return void;
end fn.main
