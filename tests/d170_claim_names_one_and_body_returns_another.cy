// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading, $b: reading] returns Lend from $a reading:
    return Lend $b;
end fn first

fn.main [none] returns void:
    return void;
end fn.main
