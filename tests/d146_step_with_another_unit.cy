// trace: D146

fam.unit length:
    length.meters: base;
end fam.unit length

fn.main [none] returns void:
    let $by: i32[unit: length.meters] = 2 length.meters;
    for $i in range[from: 1, to: 5, step_by: $by]:
        return void;
    end for
    return void;
end fn.main
