// trace: D209
@config(default);

fn half [$n: i32] -> i32:
    fails [odd];
    return $n;
end fn half

fn.main [] -> void:
    return void;
end fn.main
