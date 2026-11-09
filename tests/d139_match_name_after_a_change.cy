// The name views what's matched, so it can't be read once that's changed.
// trace: D139, D104
@config(default);

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 1];
    match $s:
        case status.failed $f:
            change $s = status.ok;
            let $n: i32 = $f.code;
        case status.ok:
            let $m: i32 = 0;
    end match
    return void;
end fn.main
