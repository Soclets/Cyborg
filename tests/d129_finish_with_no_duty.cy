// trace: D129

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    finish $b;
    return void;
end fn.main
