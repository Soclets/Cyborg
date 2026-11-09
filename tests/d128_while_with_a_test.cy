// Where the test is allowed besides `if` isn't decided.
// trace: D128, D9
@config(default);

union.tagged status:
    ok;
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    while $s: status.ok:
        return void;
    end while
    return void;
end fn.main
