// A function changing its own copy, while you expect yours to change, can't compile.
// trace: D184, D125
@config(default);

fn bump [$n: i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [] -> void:
    return void;
end fn.main
