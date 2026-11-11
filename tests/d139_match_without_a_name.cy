// A case's data is read through a name now, and nothing says the old way still works.
// trace: D139, D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.failed[code set_to 1];
    match $s:
        case status.failed:
            let $n: i32 set_to $s.code;
        case status.ok:
            let $n: i32 set_to 0;
    end match
    return void;
end fn.main
