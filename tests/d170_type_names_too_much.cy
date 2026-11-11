// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading] returns Lend from $a reading:
    return Lend $a;
end fn first

fn.main [none] returns void:
    let $x: reading = reading[celsius = 21];
    let $y: reading = reading[celsius = 25];
    let $w: Lend from <$x, $y> reading = first(Lend $x);
    return void;
end fn.main
