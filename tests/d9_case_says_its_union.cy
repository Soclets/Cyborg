// trace: D9, D33

union.tagged status:
    ok;
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.ok;
    match $s:
        case ok:
            return void;
    end match
    return void;
end fn.main
