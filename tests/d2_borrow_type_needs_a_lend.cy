// A plain name holds a value, and a lent one needs the borrow's type.
// trace: D2

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 21];
    let $b: reading set_to Lend $r;
    return void;
end fn.main
