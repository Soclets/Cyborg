// trace: D125, D20
struct box:
    n: i32[min: 0, max: 10];
end struct box

fn.main [] -> void:
    let mut $b: box = box[n = 1];
    change $b.n = 11;
    return void;
end fn.main
