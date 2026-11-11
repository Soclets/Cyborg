// trace: D125, D83

struct box:
    n: i32;
end struct box

const $b: box set_to box[n set_to 1];

fn.main [none] returns void:
    change $b.n set_to 2;
    return void;
end fn.main
