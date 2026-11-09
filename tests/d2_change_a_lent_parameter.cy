// A bare parameter is read-only.
// trace: D2, D3
@config(default);

struct box:
    n: i32;
end struct box

fn replace [$b: box] -> void:
    change $b = box[n = 9];
    return void;
end fn replace

fn.main [] -> void:
    return void;
end fn.main
