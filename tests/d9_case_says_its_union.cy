// trace: D9, D33
@config(default);

union.tagged status:
    ok
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    match $s:
        case ok:
            return void;
    end match
    return void;
end fn.main
