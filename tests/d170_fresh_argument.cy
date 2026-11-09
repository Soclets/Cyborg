// The result would borrow a value that ends with the call.
// trace: D170
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading] -> Lend from $a reading:
    return Lend $a;
end fn first

fn.main [] -> void:
    let $w: Lend from $x reading = first(reading[celsius = 1]);
    return void;
end fn.main
