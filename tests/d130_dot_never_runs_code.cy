// trace: D130, D8

struct box:
    n: i32;
end struct box

fn box.%f [$b: box] returns i32:
    return $b.n;
end fn box.%f

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    let $n: i32 set_to $b.f();
    return void;
end fn.main
