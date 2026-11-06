// Going through items that are structs isn't decided.
// trace: D2
import std;

struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let $xs: std::list<box>[len.max: inf items] = [box[n = 1]];
    for $b in $xs:
        std::println($b.n as str);
    end for
    return void;
end fn.main
