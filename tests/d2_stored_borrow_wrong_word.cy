// trace: D2

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $a: reading set_to reading[celsius set_to 1];
    let $b: Lend from $a reading set_to Copy $a;
    return void;
end fn.main
