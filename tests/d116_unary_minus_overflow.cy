// trace: D116, D42
@config(default);

fn.main [] -> void:
    let $x: i32 = -2147483648;
    let $y: i32 = -$x;
    return void;
end fn.main
