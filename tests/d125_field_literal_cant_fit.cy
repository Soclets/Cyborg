// trace: D125, D20

struct box:
    n: i32[min: 0, max: 10];
end struct box

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    change $b.n set_to 11;
    return void;
end fn.main
