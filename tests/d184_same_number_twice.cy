// trace: D184, D2
@config(default);

fn both [$a: Lent.mut i32, $b: Lent.mut i32] -> void:
    return void;
end fn both

fn.main [] -> void:
    let mut $n: i32 = 1;
    both(Lend.mut $n, Lend.mut $n);
    return void;
end fn.main
