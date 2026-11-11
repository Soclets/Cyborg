// trace: D169, D170

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn first [$a: reading] returns Lend.held from $a reading:
    return Lend $a;
end fn first

fn.main [none] returns void:
    return void;
end fn.main
