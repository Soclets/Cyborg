// `Lend.mut` means nothing else uses it meanwhile, though this read is over before the call.
// trace: D130, D2

struct box:
    n: i32;
end struct box

fn box.%%add [$b: Lent.mut box, $by: i32] returns void:
    labels [$by as by];

    change $b.n set_to $b.n + $by;
    return void;
end fn box.%%add

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    $b.%%add(by: $b.n);
    return void;
end fn.main
