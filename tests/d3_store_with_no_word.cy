// trace: D3

struct box:
    n: i32;
end struct box

fn.main [none] returns void:
    let $a: box set_to box[n set_to 1];
    let $b: box set_to $a;
    return void;
end fn.main
