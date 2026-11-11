// `$i` follows the range, and the step leaves no mark on it.
// trace: D146

fn.main [none] returns void:
    let $by: i64 set_to 1;
    let $from: i32 set_to 1;
    for $i in range[from: $from, to: 3, step_by: $by]:
        let $wrong: i64 set_to $i;
    end for
    return void;
end fn.main
