// trace: D2

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 21];
    let $b: Lend from program reading set_to Lend $r;
    return void;
end fn.main
