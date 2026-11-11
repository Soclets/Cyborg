// A function changing its own copy, while you expect yours to change, can't compile.
// trace: D184, D125

fn bump [$n: i32] returns void:
    change $n set_to $n + 1;
    return void;
end fn bump

fn.main [none] returns void:
    return void;
end fn.main
