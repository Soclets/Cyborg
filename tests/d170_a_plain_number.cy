// trace: D170, D123

struct reading:
    celsius: i32;
end struct reading

fn degrees [$a: reading] returns Lend from $a i32:
    return Lend $a.celsius;
end fn degrees

fn.main [none] returns void:
    return void;
end fn.main
