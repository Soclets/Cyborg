// Failures are their own track, so they aren't part of the return type.
// trace: D209

fn half [$n: i32] returns result<i32, odd>:
    return $n;
end fn half

fn.main [none] returns void:
    return void;
end fn.main
