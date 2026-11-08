// trace: D169
struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.held.mut from $r reading = Lend.mut $r;
    return void;
end fn.main
