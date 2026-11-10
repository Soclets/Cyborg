// trace: D171

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    let $r: reading<`text`: $a> = reading[celsius = 2];
    return void;
end fn.main
