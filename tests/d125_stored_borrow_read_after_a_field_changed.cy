// trace: D125, D2

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    let $l: Lend from $b box set_to Lend $b;
    change $b.n set_to 2;
    let $n: i32 set_to $l.n;
    return void;
end fn.main
