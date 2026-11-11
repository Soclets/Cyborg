// Whether a `fn.pure` may allocate isn't decided.
// trace: D86

fn.pure f [$n: i32] returns str:
    return $n as str;
end fn.pure f

fn.main [none] returns void:
    return void;
end fn.main
