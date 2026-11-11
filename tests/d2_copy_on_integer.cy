// trace: D2

fn f [$n: i32] returns i32:
    return $n;
end fn f

fn.main [none] returns void:
    let $n: i32 = 1;
    let $m: i32 = f(Copy $n);
    return void;
end fn.main
