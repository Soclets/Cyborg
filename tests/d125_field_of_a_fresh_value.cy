// trace: D125
@config(default);

struct box:
    n: i32;
end struct box

fn make [] -> box:
    return box[n = 1];
end fn make

fn.main [] -> void:
    change make().n = 2;
    return void;
end fn.main
