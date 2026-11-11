// Where the test is allowed besides `if` isn't decided.
// trace: D128, D9

union.tagged status:
    ok;
end union.tagged status

fn.main [none] -> void:
    let $s: status = status.ok;
    while $s: status.ok:
        return void;
    end while
    return void;
end fn.main
