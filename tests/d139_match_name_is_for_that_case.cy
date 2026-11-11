// trace: D139, D33

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.ok;
    match $s:
        case status.ok $o:
            let $n: i32 set_to $o.code;
        case status.failed:
            let $m: i32 set_to 0;
    end match
    return void;
end fn.main
