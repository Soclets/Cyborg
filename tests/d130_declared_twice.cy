// trace: D130

struct box:
    n: i32;
end struct box

fn box.%f [$b: box] -> i32:
    return $b.n;
end fn box.%f

fn box.%f [$b: box] -> i32:
    return $b.n;
end fn box.%f

fn.main [none] -> void:
    return void;
end fn.main
