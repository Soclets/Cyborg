// The fixed spelling is `fn.pure.baked`, and the other order is refused with it as the fix.
// trace: D223, D91

fn.baked.pure twice [$n: i32] returns i32:
    return $n x 2;
end fn.baked.pure twice

fn.main [none] returns void:
    return void;
end fn.main
