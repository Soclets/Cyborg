// trace: D116
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a bit_and 3 bit_or 4;
    return void;
end fn.main
