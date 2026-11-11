// trace: D184, D3

fn bump [$n: Lent.mut i32] returns void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [none] returns void:
    let $count: i32 = 1;
    bump(Lend.mut $count);
    return void;
end fn.main
