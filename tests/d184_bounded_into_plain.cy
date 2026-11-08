// Changing through a plain `i32` could break the bound, so the types must match.
// trace: D184, D20
fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [] -> void:
    let mut $n: i32[min: 0, max: 9] = 1;
    bump(Lend.mut $n);
    return void;
end fn.main
