// trace: D116
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = 1 + $a shl 2;
    return void;
end fn.main
