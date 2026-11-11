// The only way out goes through the test, but trust is for inside its block.
// trace: D128, D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.failed[code set_to 1];
    forever:
        if $s: status.failed:
            break;
        end if
    end forever
    let $n: i32 set_to $s.code;
    return void;
end fn.main
