// trace: D149, D2

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn look [$r: Lent reading] -> void:
    let $w: wrapped = Give $r as wrapped;
    return void;
end fn look

fn.main [none] -> void:
    let $a: reading = reading[celsius = 1];
    look(Lend $a);
    return void;
end fn.main
