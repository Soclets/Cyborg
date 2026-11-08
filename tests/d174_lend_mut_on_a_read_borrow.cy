// trace: D174, D2
struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $v: Lend from $r reading = Lend $r;
    let $n: Lend.mut from $r reading = Lend.mut $v;
    return void;
end fn.main
