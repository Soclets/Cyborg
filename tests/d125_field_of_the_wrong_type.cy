// trace: D125, D7

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let mut $b: box set_to box[n set_to 1];
    change $b.n set_to "two";
    return void;
end fn.main
