// Failures are their own track, so they aren't part of the return type.
// trace: D209
@config(default);

fn half [$n: i32] -> result<i32, odd>:
    return $n;
end fn half

fn.main [] -> void:
    return void;
end fn.main
