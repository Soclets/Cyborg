// Knowing the range proves nothing here, since it goes past the array, so the check stays.
// trace: D147
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::array<i32>[len: 2 items] set_to [4, 5];
    for $i in range[from: 1 items, to: 3 items]:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs[pos: $i]);
    end for
    return void;
end fn.main
