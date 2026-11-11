// trace: D184, D123

fn bump [$n: Lent.mut i32] returns void:
    change $n set_to $n + 1;
    return void;
end fn bump

fn.main [none] returns void:
    let mut $count: i32 set_to 1;
    bump(Lend $count);
    return void;
end fn.main
