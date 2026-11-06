// trace: D2
struct reading:
    celsius: i32;
end struct reading

fn replace [$r: Lent.mut reading] -> void:
    change $r = reading[celsius = 0];
    return void;
end fn replace

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21];
    let $b: Lend.mut from $r reading = Lend.mut $r;
    replace(Lend.mut $b);
    return void;
end fn.main
