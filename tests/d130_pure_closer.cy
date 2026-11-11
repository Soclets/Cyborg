// trace: D130, D121

struct box:
    n: i32;
end struct box

fn.pure box.%f [$b: box] returns i32:
    return $b.n;
end fn box.%f

fn.main [none] returns void:
    return void;
end fn.main
