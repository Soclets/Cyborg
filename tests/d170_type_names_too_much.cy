// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading] returns Lend from $a reading:
    return Lend $a;
end fn first

fn.main [none] returns void:
    let $x: reading set_to reading[celsius set_to 21];
    let $y: reading set_to reading[celsius set_to 25];
    let $w: Lend from <$x, $y> reading set_to first(Lend $x);
    return void;
end fn.main
