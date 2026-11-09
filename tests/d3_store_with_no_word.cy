// trace: D3
@config(default);

struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let $a: box = box[n = 1];
    let $b: box = $a;
    return void;
end fn.main
