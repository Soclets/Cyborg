// A `u8` can't hold -1, and a step isn't stored, so it doesn't have to.
// trace: D146
import std;

fn.main [none] -> void:
    let $from: u8 = 3;
    let $to: u8 = 0;
    for $i in range[from: $from, to: $to, step_by: -1]:
        std::io::fmt_print("{}{newln}", $i);
    end for
    return void;
end fn.main
