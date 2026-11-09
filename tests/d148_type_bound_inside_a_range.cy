// trace: D148
@config(default);

fn.main [] -> void:
    let $start: i32 = 1;
    for $i in range[from: $start: i32[min: 1, max: 3], to: 5]:
        return void;
    end for
    return void;
end fn.main
