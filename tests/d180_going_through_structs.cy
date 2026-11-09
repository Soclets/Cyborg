// A bare `for` reads items of any type in place.
// trace: D180, D182, D2
@config(default);
import std;

struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let $xs: std::list<box>[len.max: inf items] = [box[n = 1]];
    for $b in $xs:
        std::io::fmt_print("{}{newln}", $b.n);
    end for
    return void;
end fn.main
