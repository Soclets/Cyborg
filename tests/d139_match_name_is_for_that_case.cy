// trace: D139, D33
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    match $s:
        case status.ok $o:
            let $n: i32 = $o.code;
        case status.failed:
            let $m: i32 = 0;
    end match
    return void;
end fn.main
