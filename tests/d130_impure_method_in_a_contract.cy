// trace: D130, D86

struct box:
    n: i32;
end struct box

fn box.%positive [$b: box] -> bool:
    return $b.n above 0;
end fn box.%positive

fn take [$b: box] -> i32:
    requires $b.%positive();

    return $b.n;
end fn take

fn.main [none] -> void:
    return void;
end fn.main
