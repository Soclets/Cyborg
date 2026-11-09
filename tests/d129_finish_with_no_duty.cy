// trace: D129
@config(default);

struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let $b: box = box[n = 1];
    finish $b;
    return void;
end fn.main
