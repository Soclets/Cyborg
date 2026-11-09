// trace: D146
@config(default);
import std;

fn.main [] -> void:
    let $by: u8 = 3;
    for $i in range[from: -4, to: 4, step_by: $by]:
        std::io::fmt_print("{}{newln}", $i);
    end for
    return void;
end fn.main
