// `$f` is a view, so like a stored borrow it ends at its last use.
// trace: D169, D139, D2

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $s: status set_to status.failed[code set_to 1];
    match $s:
        case status.failed $f:
            let $n: i32 set_to $f.code;
            change $s set_to status.ok;
        case status.ok:
            let $m: i32 set_to 0;
    end match
    return void;
end fn.main
