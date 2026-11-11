// A bare `for` reads items of any type in place.
// trace: D180, D182, D2
import std;
import os;

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::list<box>[len.max: inf items] set_to [box[n set_to 1]];
    for $b in $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.n);
    end for
    return void;
end fn.main
