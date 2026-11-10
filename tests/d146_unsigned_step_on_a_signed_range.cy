// trace: D146
import std;

fn.main [none] -> void:
    let $by: u8 = 3;
    for $i in range[from: -4, to: 4, step_by: $by]:
        std::io::fmt_print("{}{newln}", $i);
    end for
    return void;
end fn.main
