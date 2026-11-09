// trace: D146
@config(default);
import std;

fn.main [] -> void:
    let $by: i64 = 2;
    let $from: i32 = 1;
    let $to: i32 = 5;
    for $i in range[from: $from, to: $to, step_by: $by]:
        let $kept: i32 = $i;
        std::io::fmt_print("{}{newln}", $kept);
    end for
    return void;
end fn.main
