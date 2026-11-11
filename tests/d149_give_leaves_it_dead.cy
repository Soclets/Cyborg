// trace: D149, D3

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 1];
    let $w: wrapped set_to Give $r as wrapped;
    let $again: reading set_to Copy $r;
    return void;
end fn.main
