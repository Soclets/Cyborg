// trace: D57

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 21];
    let $n: i32 set_to $r.kelvin;
    return void;
end fn.main
