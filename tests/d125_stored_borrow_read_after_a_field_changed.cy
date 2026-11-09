// trace: D125, D2
@config(default);

struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let mut $b: box = box[n = 1];
    let $l: Lend from $b box = Lend $b;
    change $b.n = 2;
    let $n: i32 = $l.n;
    return void;
end fn.main
