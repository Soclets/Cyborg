// trace: D139, D3

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $s: status set_to status.failed[code set_to 1];
    match $s:
        case status.failed $f:
            change $f set_to status.ok;
        case status.ok:
            let $m: i32 set_to 0;
    end match
    return void;
end fn.main
