// trace: D148, D20
@config(default);
import std;

fn.main [] -> void:
    let $start: i32[min: 1, max: 3] = 2;
    for $i in range[from: $start, to: 3]:
        std::io::fmt_print("{}{newln}", $i);
    end for
    return void;
end fn.main
