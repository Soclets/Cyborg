// trace: D84, D76

fn half [$n: i32] returns i32:
    precond => $n at_least 0;
    return $n;
end fn half

fn.main [none] returns void:
    return void;
end fn.main
