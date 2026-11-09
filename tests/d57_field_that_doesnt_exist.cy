// trace: D57
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $r: reading = reading[celsius = 21];
    let $n: i32 = $r.kelvin;
    return void;
end fn.main
