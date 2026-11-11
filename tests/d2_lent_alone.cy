// trace: D2

struct box:
    n: i32;
end struct box

fn f [$b: Lent box] returns i32:
    return $b.n;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
