// Whether that ends at its last use or its scope isn't decided, as for a stored borrow.
// trace: D139, D2
union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 1];
    match $s:
        case status.failed $f:
            let $n: i32 = $f.code;
            change $s = status.ok;
        case status.ok:
            let $m: i32 = 0;
    end match
    return void;
end fn.main
