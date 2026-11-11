// A bare parameter is read-only.
// trace: D2, D3

struct box:
    n: i32;
end struct box

fn replace [$b: box] returns void:
    change $b = box[n = 9];
    return void;
end fn replace

fn.main [none] returns void:
    return void;
end fn.main
