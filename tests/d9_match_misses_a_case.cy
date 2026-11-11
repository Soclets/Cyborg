// trace: D9

union.tagged status:
    ok;
    failed[code: i32];
    lost;
end union.tagged status

fn.main [none] returns void:
    let $s: status = status.ok;
    match $s:
        case status.ok:
            return void;
    end match
    return void;
end fn.main
