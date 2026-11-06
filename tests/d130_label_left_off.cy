// trace: D130, D113
struct box:
    n: i32;
end struct box

fn box.%plus [$b: box, $by: i32] -> i32:
    labels [$by as by];

    return $b.n + $by;
end fn box.%plus

fn.main [] -> void:
    let $b: box = box[n = 1];
    let $n: i32 = $b.%plus(1);
    return void;
end fn.main
