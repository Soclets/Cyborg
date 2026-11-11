// trace: D209

fn half [$n: i32] returns i32:
    fails [odd];
    return $n;
end fn half

fn.main [none] returns void:
    return void;
end fn.main
