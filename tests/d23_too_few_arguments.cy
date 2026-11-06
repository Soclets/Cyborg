// trace: D23, D113
fn f [$a: i32] -> i32:
    return $a;
end fn f

fn.main [] -> void:
    let $a: i32 = f();
    return void;
end fn.main
