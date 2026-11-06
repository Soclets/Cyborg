// trace: D130
struct box:
    n: i32;
end struct box

fn box.%f [$n: i32] -> i32:
    return $n;
end fn box.%f

fn.main [] -> void:
    return void;
end fn.main
