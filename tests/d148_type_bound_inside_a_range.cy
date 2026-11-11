// trace: D148

fn.main [none] returns void:
    let $start: i32 set_to 1;
    for $i in range[from: $start: i32[min: 1, max: 3], to: 5]:
        return void;
    end for
    return void;
end fn.main
