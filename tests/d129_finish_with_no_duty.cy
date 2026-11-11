// trace: D129

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let $b: box = box[n = 1];
    finish $b;
    return void;
end fn.main
