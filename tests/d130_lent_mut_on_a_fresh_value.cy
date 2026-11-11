// trace: D130, D4

struct box:
    n: i32;
end struct box

fn box.%%bump [$b: Lent.mut box] returns void:
    return void;
end fn box.%%bump

fn make [none] returns box:
    return box[n set_to 1];
end fn make

fn.main [none] returns void:
    make().%%bump();
    return void;
end fn.main
