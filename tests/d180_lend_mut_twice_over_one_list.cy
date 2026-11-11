// trace: D180, D2
import std;

fn.main [none] returns void:
    let mut $xs: std::array<i32>[len: 2 items] set_to [1, 2];
    for $a in Lend.mut $xs:
        for $b in Lend.mut $xs:
            change $b set_to $a;
        end for
    end for
    return void;
end fn.main
