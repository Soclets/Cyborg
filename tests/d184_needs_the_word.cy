// trace: D184, D3
fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [] -> void:
    let mut $count: i32 = 1;
    bump($count);
    return void;
end fn.main
