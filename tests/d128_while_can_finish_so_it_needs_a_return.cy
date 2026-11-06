// `while` might not run, even on `true`, because only `forever` says it has no end.
// trace: D128, D11
fn f [] -> i32:
    while true:
        return 1;
    end while
end fn f

fn.main [] -> void:
    return void;
end fn.main
