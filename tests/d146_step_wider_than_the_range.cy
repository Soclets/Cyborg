// trace: D146
import std;
@config(default);

fn.main [] -> void:
    let $by: i64 = 2;
    let $from: i32 = 1;
    let $to: i32 = 5;
    for $i in range[from: $from, to: $to, step_by: $by]:
        let $kept: i32 = $i;
        std::println($kept as str);
    end for
    return void;
end fn.main
