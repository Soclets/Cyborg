// trace: D7, D43
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i64 = 2;
    let $c: i32 = $a + $b;
    return void;
end fn.main
