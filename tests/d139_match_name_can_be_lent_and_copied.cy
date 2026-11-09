// trace: D139, D2
@config(default);
import std;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn show [$s: status] -> void:
    if $s: status.failed:
        std::io::fmt_print("{}{newln}", $s.code);
    end if
    return void;
end fn show

fn keep [$s: Given status] -> void:
    show(Lend $s);
    return void;
end fn keep

fn.main [] -> void:
    let $s: status = status.failed[code = 1];
    match $s:
        case status.failed $f:
            show(Lend $f);
            keep(Copy $f);
        case status.ok:
            let $m: i32 = 0;
    end match
    return void;
end fn.main
