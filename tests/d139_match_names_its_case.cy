// trace: D139, D9
import std;

union.tagged status:
    ok;
    failed[code: i32];
    lost[where: str];
end union.tagged status

fn show [$s: status] -> void:
    match $s:
        case status.failed $f:
            std::io::fmt_print("failed {}{newln}", $f.code);
        case status.lost $l:
            std::io::fmt_print("lost at {}{newln}", $l.where);
        case status.ok:
            std::io::fmt_print("ok{newln}");
    end match
    return void;
end fn show

fn make [none] -> status:
    return status.failed[code = 8];
end fn make

fn.main [none] -> void:
    show(status.failed[code = 7]);
    show(status.lost[where = "home"]);
    show(status.ok);
    match make():
        case status.failed $f:
            std::io::fmt_print("fresh {}{newln}", $f.code);
        case status.lost:
            std::io::fmt_print("lost{newln}");
        case status.ok:
            std::io::fmt_print("ok{newln}");
    end match
    return void;
end fn.main
