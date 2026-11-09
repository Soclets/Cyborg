// trace: D170
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn make [$a: reading] -> Lend from $a reading:
    let $r: reading = reading[celsius = 1];
    return Lend $r;
end fn make

fn.main [] -> void:
    return void;
end fn.main
