// An alias is fully interchangeable, so it builds, tests and matches too.
// trace: D134, D9, D57
import std;

struct reading:
    celsius: i32;
end struct reading

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

type.alias r = reading;
type.alias s = status;

fn.main [] -> void:
    let $a: r = r[celsius = 21];
    let $b: reading = Copy $a;
    std::println($b.celsius as str);
    let $c: s = s.failed[code = 7];
    if $c: s.failed:
        std::println($c.code as str);
    end if
    match $c:
        case s.ok:
            std::println("ok");
        case status.failed:
            std::println("failed");
    end match
    return void;
end fn.main
