// trace: D86

struct box:
    n: i32;
end struct box

fn.pure f [$b: Lent.mut box] returns i32:
    return 1;
end fn.pure f

fn.main [none] returns void:
    return void;
end fn.main
