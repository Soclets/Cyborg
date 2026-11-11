// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn keep [$a: Given reading] returns Lend from $a reading:
    return Lend $a;
end fn keep

fn.main [none] returns void:
    return void;
end fn.main
