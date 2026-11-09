// `for` reads in place, and `Lend.mut` means nothing else uses it meanwhile.
// trace: D2
@config(default);
import std;

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = [1, 2];
    for $x in $xs:
        $xs.std::%%append(val: $x);
    end for
    return void;
end fn.main
