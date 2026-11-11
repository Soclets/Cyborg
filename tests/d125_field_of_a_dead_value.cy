// trace: D125, D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    eat(Give $b);
    change $b.n set_to 2;
    return void;
end fn.main
