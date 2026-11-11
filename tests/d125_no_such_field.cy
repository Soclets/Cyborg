// trace: D125, D57

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    change $b.m set_to 2;
    return void;
end fn.main
