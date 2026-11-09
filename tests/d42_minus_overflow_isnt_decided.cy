// trace: D42
@config(default);

fn.main [] -> void:
    let $a: i32 = -2147483648;
    let $b: i32 = $a - 1;
    return void;
end fn.main
