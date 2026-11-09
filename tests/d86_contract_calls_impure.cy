// trace: D86
@config(default);

fn check [$n: i32] -> bool:
    return $n at_least 0;
end fn check

fn f [$n: i32] -> i32:
    requires check($n);

    return $n;
end fn f

fn.main [] -> void:
    return void;
end fn.main
