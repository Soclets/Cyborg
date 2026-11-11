// trace: D125, D57

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let mut $b: box = box[n = 1];
    change $b.m = 2;
    return void;
end fn.main
