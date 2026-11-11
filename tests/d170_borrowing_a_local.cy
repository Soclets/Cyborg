// trace: D170

struct reading:
    celsius: i32;
end struct reading

fn make [none] returns Lend from $r reading:
    let $r: reading set_to reading[celsius set_to 1];
    return Lend $r;
end fn make

fn.main [none] returns void:
    return void;
end fn.main
