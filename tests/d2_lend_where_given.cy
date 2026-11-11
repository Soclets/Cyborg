// `Given` takes `Give` or `Copy`, so no one fix can be shown.
// trace: D2, D16

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    eat(Lend $b);
    return void;
end fn.main
