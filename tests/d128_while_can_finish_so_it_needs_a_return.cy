// `while` might not run, even on `true`, because only `forever` says it has no end.
// trace: D128, D11

fn f [none] returns i32:
    while true:
        return 1;
    end while
end fn f

fn.main [none] returns void:
    return void;
end fn.main
