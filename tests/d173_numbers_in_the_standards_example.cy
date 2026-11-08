// The standard's example lends two number fields with `Lend.mut`, which D123 refuses for numbers.
// trace: D173, D123
struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn adjust [$c: Lent.mut i32, $t: Lent.mut i32] -> void:
    return void;
end fn adjust

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    adjust(Lend.mut $r.celsius, Lend.mut $r.time);
    return void;
end fn.main
