// trace: D125, D2

struct box:
    n: i32;
end struct box

fn.main [none] -> void:
    let mut $b: box = box[n = 1];
    let $l: Lend from $b box = Lend $b;
    change $l.n = 2;
    return void;
end fn.main
