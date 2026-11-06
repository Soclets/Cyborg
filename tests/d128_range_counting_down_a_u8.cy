// A `u8` can't hold -1, so its own type can't say the step.
// trace: D128
fn.main [] -> void:
    let $from: u8 = 5;
    let $to: u8 = 1;
    for $i in range[from: $from, to: $to, step_by: -1]:
        return void;
    end for
    return void;
end fn.main
