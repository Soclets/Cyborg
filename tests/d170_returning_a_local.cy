// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn make [$a: reading] returns Lend from $a reading:
    let $r: reading set_to reading[celsius set_to 1];
    return Lend $r;
end fn make

fn.main [none] returns void:
    return void;
end fn.main
