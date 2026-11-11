// The name views what's matched, so it can't be read once that's changed.
// trace: D139, D104

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $s: status set_to status.failed[code set_to 1];
    match $s:
        case status.failed $f:
            change $s set_to status.ok;
            let $n: i32 set_to $f.code;
        case status.ok:
            let $m: i32 set_to 0;
    end match
    return void;
end fn.main
