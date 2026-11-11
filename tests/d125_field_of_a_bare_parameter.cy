// trace: D125

struct box:
    n: i32;
end struct box

fn f [$b: box] returns void:
    change $b.n set_to 2;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
