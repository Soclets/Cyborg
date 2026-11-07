// A `match` arm isn't a failure's arm.
// trace: D139
union.tagged status:
    ok
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    match $s:
        case status.ok:
            instead 0;
    end match
    return void;
end fn.main
