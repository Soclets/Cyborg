// Whether a `fn.pure` may fail isn't decided.
// trace: D86

fn.pure f [none] -> i32 fails [a]:
    return 1;
end fn.pure f

fn.main [none] -> void:
    return void;
end fn.main
