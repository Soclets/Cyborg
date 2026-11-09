// trace: D84, D76
@config(default);

fn half [$n: i32] -> i32:
    precond => $n at_least 0;
    return $n;
end fn half

fn.main [] -> void:
    return void;
end fn.main
