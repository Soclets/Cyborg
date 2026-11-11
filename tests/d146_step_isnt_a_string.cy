// trace: D146, D128

fn.main [none] returns void:
    let $by: str set_to "1";
    for $i in range[from: 1, to: 5, step_by: $by]:
        return void;
    end for
    return void;
end fn.main
