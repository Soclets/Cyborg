// `change` on a field isn't in the standard yet.
// trace: D3
struct box:
    n: i32;
end struct box

fn.main [] -> void:
    let mut $b: box = box[n = 1];
    change $b.n = 2;
    return void;
end fn.main
