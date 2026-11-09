// trace: D2
@config(default);

fn f [$n: i32] -> i32:
    return $n;
end fn f

fn.main [] -> void:
    let $n: i32 = 1;
    let $m: i32 = f(Copy $n);
    return void;
end fn.main
