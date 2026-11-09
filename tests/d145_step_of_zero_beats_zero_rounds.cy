// This range couldn't reach its end anyway, but a step of 0 is a failure either way.
// trace: D145
@config(default);

fn.main [] -> void:
    let $by: i32 = 0;
    let $start: i32 = 5;
    for $i in range[from: $start, to: 1, step_by: $by]:
        return void;
    end for
    return void;
end fn.main
