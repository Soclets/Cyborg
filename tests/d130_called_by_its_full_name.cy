// trace: D130
@config(default);

struct box:
    n: i32;
end struct box

fn box.%read [$b: box] -> i32:
    return $b.n;
end fn box.%read

fn.main [] -> void:
    let $b: box = box[n = 1];
    let $n: i32 = box.%read(Lend $b);
    return void;
end fn.main
