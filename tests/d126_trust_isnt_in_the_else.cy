// Inside the block the compiler trusts it, and `else:` isn't inside it, even with two cases.
// trace: D126, D9
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    if $s: status.ok:
        return void;
    else:
        let $n: i32 = $s.code;
    end if
    return void;
end fn.main
