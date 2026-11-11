// trace: D174, D2

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn eat [$r: Given reading] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    let $m: Lend.mut from $r reading set_to Lend.mut $r;
    eat(Give $m);
    return void;
end fn.main
