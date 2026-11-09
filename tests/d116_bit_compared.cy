// trace: D116
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: bool = $a bit_and 1 == 1;
    return void;
end fn.main
