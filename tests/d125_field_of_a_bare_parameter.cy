// trace: D125

struct box:
    n: i32;
end struct box

fn f [$b: Lent box] -> void:
    change $b.n = 2;
    return void;
end fn f

fn.main [none] -> void:
    return void;
end fn.main
