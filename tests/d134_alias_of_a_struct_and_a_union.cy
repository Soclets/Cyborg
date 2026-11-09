// An alias is fully interchangeable, so it builds, tests and matches too.
// trace: D134, D9, D57
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

type.alias r = reading;
type.alias s = status;

fn.main [] -> void:
    let $a: r = r[celsius = 21];
    let $b: reading = Copy $a;
    std::io::fmt_print("{}{newln}", $b.celsius);
    let $c: s = s.failed[code = 7];
    if $c: s.failed:
        std::io::fmt_print("{}{newln}", $c.code);
    end if
    match $c:
        case s.ok:
            std::io::fmt_print("ok{newln}");
        case status.failed:
            std::io::fmt_print("failed{newln}");
    end match
    return void;
end fn.main
