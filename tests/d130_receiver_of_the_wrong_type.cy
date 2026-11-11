// trace: D130

struct box:
    n: i32;
end struct box

fn box.%f [$n: i32] returns i32:
    return $n;
end fn box.%f

fn.main [none] returns void:
    return void;
end fn.main
