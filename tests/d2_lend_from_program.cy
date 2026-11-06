// trace: D2
struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $r: reading = reading[celsius = 21];
    let $b: Lend from program reading = Lend $r;
    return void;
end fn.main
