// trace: D125, D57
@config(default);

struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let mut $b: box = box[n = 1];
    change $b.m = 2;
    return void;
end fn.main
