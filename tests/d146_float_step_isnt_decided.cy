// A step takes any integer, and floats wait on whether ranges of floats exist at all.
// trace: D146
@config(default);

fn.main [] -> void:
    let $by: f64 = 1.5;
    for $i in range[from: 1, to: 5, step_by: $by]:
        return void;
    end for
    return void;
end fn.main
