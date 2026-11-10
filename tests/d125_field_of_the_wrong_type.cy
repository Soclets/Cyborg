// trace: D125, D7

struct box:
    n: i32;
end struct box

fn.main [none] -> void:
    let mut $b: box = box[n = 1];
    change $b.n = "two";
    return void;
end fn.main
