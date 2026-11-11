// trace: D3

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let $a: box = box[n = 1];
    let $b: box = $a;
    return void;
end fn.main
