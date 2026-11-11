// trace: D174

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] returns void:
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    let $m: Lend.mut from $r reading set_to Lend.mut $r;
    let $n: Lend.mut from $m reading set_to Lend.mut $m;
    return void;
end fn.main
