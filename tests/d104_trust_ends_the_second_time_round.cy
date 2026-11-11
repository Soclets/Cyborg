// trace: D104
import std;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        for $x in $xs:
            let $b: i32 = $s.code;
            change $s = status.ok;
        end for
    end if
    return void;
end fn.main
