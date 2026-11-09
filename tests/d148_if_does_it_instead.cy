// trace: D148, D20
import std;
@config(default);

fn.main [] -> void:
    let $start: i32 = 150;
    if $start: i32[min: 100, max: 200]:
        for $i in range[from: $start, to: 151]:
            std::io::fmt_print("{}{newln}", $i);
        end for
    end if
    return void;
end fn.main
