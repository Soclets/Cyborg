// trace: D42
@config(default);

fn.main [] -> void:
    let $a: i32 = 65536;
    let $b: i32 = $a x $a;
    return void;
end fn.main
