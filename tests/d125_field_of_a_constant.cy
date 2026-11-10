// trace: D125, D83

struct box:
    n: i32;
end struct box

const $b: box = box[n = 1];

fn.main [none] -> void:
    change $b.n = 2;
    return void;
end fn.main
