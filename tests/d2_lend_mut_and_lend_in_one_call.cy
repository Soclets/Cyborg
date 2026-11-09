// trace: D2
@config(default);

struct box:
    n: i32;
end struct box

fn both [$a: Lent.mut box, $b: box] -> void:
    return void;
end fn both

fn.main [] -> void:
    let mut $b: box = box[n = 1];
    both(Lend.mut $b, Lend $b);
    return void;
end fn.main
