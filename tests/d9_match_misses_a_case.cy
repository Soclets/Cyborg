// trace: D9
@config(default);

union.tagged status:
    ok
    failed[code: i32]
    lost
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    match $s:
        case status.ok:
            return void;
    end match
    return void;
end fn.main
