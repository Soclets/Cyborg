// trace: D169, D223

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] returns void:
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    let $lock: Lend.held.mut from $r reading set_to Lend.mut $r;
    return void;
end fn.main
