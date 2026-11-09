// trace: D139, D3
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 1];
    match $s:
        case status.failed $f:
            change $f = status.ok;
        case status.ok:
            let $m: i32 = 0;
    end match
    return void;
end fn.main
