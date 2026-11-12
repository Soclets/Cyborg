// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn first [$a: Lent reading, $b: Lent reading] -> Lend from $a reading:
    return Lend $b;
end fn first

fn.main [none] -> void:
    return void;
end fn.main
