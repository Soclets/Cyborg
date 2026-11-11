// trace: D130, D113

struct box:
    n: i32;
end struct box

fn box.%f [$b: box] returns i32:
    labels [$b as thing];

    return $b.n;
end fn box.%f

fn.main [none] returns void:
    return void;
end fn.main
