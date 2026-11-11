// trace: D130, D113

struct box:
    n: i32;
end struct box

fn box.%plus [$b: box, $by: i32] returns i32:
    labels [$by as by];

    return $b.n + $by;
end fn box.%plus

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    let $n: i32 set_to $b.%plus(1);
    return void;
end fn.main
