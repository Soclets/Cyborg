// A written 0 can't reach any end, even when the other numbers are only known while running.
// trace: D145, D128
fn.main [] -> void:
    let $start: i32 = 1;
    for $i in range[from: $start, to: 5, step_by: 0]:
        return void;
    end for
    return void;
end fn.main
