// trace: D130, D91
struct box:
    n: i32;
end struct box

fn.pure box.%twice [$b: box] -> i32:
    return $b.n x 2;
end fn.pure box.%twice

const $b: box = box[n = 4];
const $c: i32 = $b.%twice();

fn.main [] -> void:
    return void;
end fn.main
