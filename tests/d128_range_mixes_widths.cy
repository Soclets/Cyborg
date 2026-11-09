// trace: D128, D7
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i64 = 2;
    for $i in range[from: $a, to: $b]:
        return void;
    end for
    return void;
end fn.main
