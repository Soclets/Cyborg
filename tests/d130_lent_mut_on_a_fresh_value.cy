// trace: D130, D4

struct box:
    n: i32;
end struct box

fn box.%%bump [$b: Lent.mut box] -> void:
    return void;
end fn box.%%bump

fn make [none] -> box:
    return box[n = 1];
end fn make

fn.main [none] -> void:
    make().%%bump();
    return void;
end fn.main
