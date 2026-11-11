// trace: D125

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    change $b.n set_to 2;
    return void;
end fn.main
