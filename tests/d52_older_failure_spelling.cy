// trace: D47, D52
@config(default);

fn half [$n: i32] -> i32 or fail[odd]:
    return $n;
end fn half

fn.main [] -> void:
    return void;
end fn.main
