// trace: D184, D2

fn both [$a: Lent.mut i32, $b: Lent.mut i32] returns void:
    return void;
end fn both

fn.main [none] returns void:
    let mut $n: i32 = 1;
    both(Lend.mut $n, Lend.mut $n);
    return void;
end fn.main
