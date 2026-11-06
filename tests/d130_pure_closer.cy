// trace: D130, D121
struct box:
    n: i32;
end struct box

fn.pure box.%f [$b: box] -> i32:
    return $b.n;
end fn box.%f

fn.main [] -> void:
    return void;
end fn.main
