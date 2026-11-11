// trace: D130, D91

struct box:
    n: i32;
end struct box

fn.pure box.%twice [$b: box] returns i32:
    return $b.n x 2;
end fn.pure box.%twice

const $b: box set_to box[n set_to 4];
const $c: i32 set_to $b.%twice();

fn.main [none] returns void:
    return void;
end fn.main
