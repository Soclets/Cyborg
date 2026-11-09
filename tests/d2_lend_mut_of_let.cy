// trace: D2, D3
@config(default);

struct box:
    n: i32;
end struct box

fn replace [$b: Lent.mut box] -> void:
    return void;
end fn replace

fn.main [] -> void:
    let $b: box = box[n = 1];
    replace(Lend.mut $b);
    return void;
end fn.main
