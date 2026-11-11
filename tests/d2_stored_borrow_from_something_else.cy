// trace: D2

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    let $c: reading = reading[celsius = 2];
    let $b: Lend from $a reading = Lend $c;
    return void;
end fn.main
