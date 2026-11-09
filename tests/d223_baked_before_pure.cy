// The fixed spelling is `fn.pure.baked`, and the other order is refused with it as the fix.
// trace: D223, D91
@config(default);

fn.baked.pure twice [$n: i32] -> i32:
    return $n x 2;
end fn.baked.pure twice

fn.main [] -> void:
    return void;
end fn.main
