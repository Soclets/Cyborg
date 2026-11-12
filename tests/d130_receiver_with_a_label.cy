// trace: D130, D113

struct box:
    n: i32;
end struct box

fn box.%f [$b: Lent box] -> i32:
    labels [$b as thing];

    return $b.n;
end fn box.%f

fn.main [none] -> void:
    return void;
end fn.main
