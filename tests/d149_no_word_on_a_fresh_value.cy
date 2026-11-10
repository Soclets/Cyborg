// trace: D149, D2

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn make [none] -> reading:
    return reading[celsius = 3];
end fn make

fn.main [none] -> void:
    let $w: wrapped = Give make() as wrapped;
    return void;
end fn.main
