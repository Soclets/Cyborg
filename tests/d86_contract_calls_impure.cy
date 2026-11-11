// trace: D86

fn check [$n: i32] returns bool:
    return $n at_least 0;
end fn check

fn f [$n: i32] returns i32:
    requires check($n);

    return $n;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
