// trace: D170
struct other:
    n: i32;
end struct other

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading] -> Lend from $a reading:
    return Lend $a;
end fn first

fn.main [] -> void:
    let $x: reading = reading[celsius = 4];
    let $w: Lend from $x other = first(Lend $x);
    return void;
end fn.main
