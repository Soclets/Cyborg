// trace: D130, D113

struct box:
    n: i32;
end struct box

fn box.%read [$b: box] returns i32:
    return $b.n;
end fn box.%read

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    let $n: i32 set_to $b.%read(1);
    return void;
end fn.main
