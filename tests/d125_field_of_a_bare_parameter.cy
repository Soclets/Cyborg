// trace: D125
struct box:
    n: i32;
end struct box

fn f [$b: box] -> void:
    change $b.n = 2;
    return void;
end fn f

fn.main [] -> void:
    return void;
end fn.main
