// trace: D2
struct box:
    n: i32;
end struct box

fn f [$b: Lent box] -> i32:
    return $b.n;
end fn f

fn.main [] -> void:
    return void;
end fn.main
