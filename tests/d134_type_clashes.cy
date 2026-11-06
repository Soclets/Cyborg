// trace: D134, D75
fn type [] -> i32:
    return 1;
end fn type

fn.main [] -> void:
    let $a: i32 = type();
    return void;
end fn.main
