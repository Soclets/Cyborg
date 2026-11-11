// trace: D2, D3

struct box:
    n: i32;
end struct box

fn replace [$b: Lent.mut box] returns void:
    return void;
end fn replace

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    replace(Lend.mut $b);
    return void;
end fn.main
