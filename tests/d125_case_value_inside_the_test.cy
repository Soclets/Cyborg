// D125 speaks of fields, and whether the test's trust survives this isn't stated.
// trace: D125, D104

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $s: status set_to status.failed[code set_to 1];
    if $s: status.failed:
        change $s.code set_to 2;
    end if
    return void;
end fn.main
