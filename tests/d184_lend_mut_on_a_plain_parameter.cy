// trace: D184, D123
fn show [$n: i32] -> void:
    return void;
end fn show

fn.main [] -> void:
    let mut $count: i32 = 1;
    show(Lend.mut $count);
    return void;
end fn.main
