// trace: D125, D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let mut $b: box = box[n = 1];
    eat(Give $b);
    change $b.n = 2;
    return void;
end fn.main
