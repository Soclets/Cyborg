// `Given` isn't `let mut` or `Lent.mut`, so the standard's rule doesn't reach it.
// trace: D125

struct box:
    n: i32;
end struct box

fn f [$b: Given box] returns void:
    change $b.n = 2;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
