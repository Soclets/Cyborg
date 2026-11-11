// trace: D125

struct box:
    n: i32;
end struct box

fn make [none] returns box:
    return box[n set_to 1];
end fn make

fn.main [none] returns void:
    change make().n set_to 2;
    return void;
end fn.main
