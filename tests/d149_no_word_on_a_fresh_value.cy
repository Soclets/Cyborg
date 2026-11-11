// trace: D149, D2

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn make [none] returns reading:
    return reading[celsius set_to 3];
end fn make

fn.main [none] returns void:
    let $w: wrapped set_to Give make() as wrapped;
    return void;
end fn.main
