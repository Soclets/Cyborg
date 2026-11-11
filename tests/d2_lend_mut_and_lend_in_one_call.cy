// trace: D2

struct box:
    n: i32;
end struct box

fn both [$a: Lent.mut box, $b: box] returns void:
    return void;
end fn both

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    both(Lend.mut $b, Lend $b);
    return void;
end fn.main
