// Whether a `fn.pure` may fail isn't decided.
// trace: D86
@config(default);

fn.pure f [] -> i32 fails [a]:
    return 1;
end fn.pure f

fn.main [] -> void:
    return void;
end fn.main
