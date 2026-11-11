// Whether a `fn.pure` may fail isn't decided.
// trace: D86

fn.pure f [none] returns i32 fails [a]:
    return 1;
end fn.pure f

fn.main [none] returns void:
    return void;
end fn.main
