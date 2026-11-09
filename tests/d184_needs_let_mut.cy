// trace: D184, D3
@config(default);

fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [] -> void:
    let $count: i32 = 1;
    bump(Lend.mut $count);
    return void;
end fn.main
