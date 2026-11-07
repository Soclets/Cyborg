// D139 only shows a name on `case`, and the test trusts `$s` itself.
// trace: D139, D9
union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    if $s: status.failed $f:
        let $n: i32 = $f.code;
    end if
    return void;
end fn.main
