// trace: D171
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $a: reading = reading[celsius = 1];
    let $r: reading<`text`: $a> = reading[celsius = 2];
    return void;
end fn.main
