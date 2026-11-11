// trace: D184

fn bump [$n: Lent.mut i32] returns void:
    change $n set_to $n + 1;
    return void;
end fn bump

fn.main [none] returns void:
    bump(5);
    return void;
end fn.main
