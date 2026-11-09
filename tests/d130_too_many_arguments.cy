// trace: D130, D113
@config(default);

struct box:
    n: i32;
end struct box

fn box.%read [$b: box] -> i32:
    return $b.n;
end fn box.%read

fn.main [] -> void:
    let $b: box = box[n = 1];
    let $n: i32 = $b.%read(1);
    return void;
end fn.main
