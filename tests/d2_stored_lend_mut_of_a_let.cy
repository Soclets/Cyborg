// trace: D2, D3

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 21];
    let $b: Lend.mut from $r reading set_to Lend.mut $r;
    return void;
end fn.main
