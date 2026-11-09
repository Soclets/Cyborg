// trace: D52
@config(default);

fn half [$n: i32] -> i32 fails [odd]:
    fail negative;
end fn half

fn.main [] -> void:
    return void;
end fn.main
