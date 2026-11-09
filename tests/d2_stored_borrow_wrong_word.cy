// trace: D2
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $a: reading = reading[celsius = 1];
    let $b: Lend from $a reading = Copy $a;
    return void;
end fn.main
