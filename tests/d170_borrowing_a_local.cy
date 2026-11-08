// trace: D170
struct reading:
    celsius: i32;
end struct reading

fn make [] -> Lend from $r reading:
    let $r: reading = reading[celsius = 1];
    return Lend $r;
end fn make

fn.main [] -> void:
    return void;
end fn.main
