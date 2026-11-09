// trace: D23
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    let $c: bool = $a<$b;
    return void;
end fn.main
