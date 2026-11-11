// trace: D125, D2

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    let $l: Lend from $b box set_to Lend $b;
    change $l.n set_to 2;
    return void;
end fn.main
