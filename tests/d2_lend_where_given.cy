// `Given` takes `Give` or `Copy`, so no one fix can be shown.
// trace: D2, D16
struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $b: box = box[n = 1];
    eat(Lend $b);
    return void;
end fn.main
