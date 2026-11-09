// trace: D125
@config(default);

struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let $b: box = box[n = 1];
    change $b.n = 2;
    return void;
end fn.main
