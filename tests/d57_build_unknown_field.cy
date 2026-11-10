// trace: D57

struct reading:
    celsius: i32;
end struct reading

fn.main [none] -> void:
    let $r: reading = reading[celsius = 21, kelvin = 300];
    return void;
end fn.main
