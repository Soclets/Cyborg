// trace: D170

struct other:
    n: i32;
end struct other

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading] returns Lend from $a reading:
    return Lend $a;
end fn first

fn.main [none] returns void:
    let $x: reading set_to reading[celsius set_to 4];
    let $w: Lend from $x other set_to first(Lend $x);
    return void;
end fn.main
