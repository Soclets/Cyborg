// trace: D149, D2

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn look [$r: reading] returns void:
    let $w: wrapped set_to Give $r as wrapped;
    return void;
end fn look

fn.main [none] returns void:
    let $a: reading set_to reading[celsius set_to 1];
    look(Lend $a);
    return void;
end fn.main
