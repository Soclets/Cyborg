// trace: D130, D113

struct box:
    n: i32;
end struct box

fn box.%read [$b: Lent box] -> i32:
    return $b.n;
end fn box.%read

fn.main [none] -> void:
    let $b: box = box[n = 1];
    let $n: i32 = $b.%read(1);
    return void;
end fn.main
