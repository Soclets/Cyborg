// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn make [$a: Lent reading] -> Lend from $a reading:
    let $r: reading = reading[celsius = 1];
    return Lend $r;
end fn make

fn.main [none] -> void:
    return void;
end fn.main
