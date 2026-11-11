// A plain name holds a value, and a lent one needs the borrow's type.
// trace: D2

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let $r: reading = reading[celsius = 21];
    let $b: reading = Lend $r;
    return void;
end fn.main
