// trace: D57

struct reading:
    celsius: i32;
    time: i64;
end struct reading

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 21];
    return void;
end fn.main
