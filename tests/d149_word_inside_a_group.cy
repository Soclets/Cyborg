// trace: D149

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [none] -> void:
    let $r: reading = reading[celsius = 1];
    let $w: wrapped = (Give $r) as wrapped;
    return void;
end fn.main
