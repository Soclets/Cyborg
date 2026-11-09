// The only way out goes through the test, but trust is for inside its block.
// trace: D128, D9
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let $s: status = status.failed[code = 1];
    forever:
        if $s: status.failed:
            break;
        end if
    end forever
    let $n: i32 = $s.code;
    return void;
end fn.main
