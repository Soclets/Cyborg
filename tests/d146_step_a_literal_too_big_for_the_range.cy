// A step isn't stored, so a `u8` range can take 300, and jumps past its end.
// trace: D146
import std;

fn.main [none] -> void:
    let $from: u8 = 1;
    let $to: u8 = 5;
    for $i in range[from: $from, to: $to, step_by: 300]:
        std::io::fmt_print("{}{newln}", $i);
    end for
    return void;
end fn.main
