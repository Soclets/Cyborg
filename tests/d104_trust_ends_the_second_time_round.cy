// trace: D104
import std;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $xs: std::array<i32>[len: 2 items] set_to [1, 2];
    let mut $s: status set_to status.failed[code set_to 3];
    if $s: status.failed:
        for $x in $xs:
            let $b: i32 set_to $s.code;
            change $s set_to status.ok;
        end for
    end if
    return void;
end fn.main
