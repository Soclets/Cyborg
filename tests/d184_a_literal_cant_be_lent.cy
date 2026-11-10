// trace: D184

fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [none] -> void:
    bump(5);
    return void;
end fn.main
