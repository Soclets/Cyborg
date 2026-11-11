// trace: D139, D9
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
    lost[where: str];
end union.tagged status

fn show [$s: status] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    match $s:
        case status.failed $f:
            std::io::fmt_print(Lend.mut $out, "failed {}{newln}", $f.code);
        case status.lost $l:
            std::io::fmt_print(Lend.mut $out, "lost at {}{newln}", $l.where);
        case status.ok:
            std::io::fmt_print(Lend.mut $out, "ok{newln}");
    end match
    return void;
end fn show

fn make [none] returns status:
    return status.failed[code = 8];
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    show(status.failed[code = 7]);
    show(status.lost[where = "home"]);
    show(status.ok);
    match make():
        case status.failed $f:
            std::io::fmt_print(Lend.mut $out, "fresh {}{newln}", $f.code);
        case status.lost:
            std::io::fmt_print(Lend.mut $out, "lost{newln}");
        case status.ok:
            std::io::fmt_print(Lend.mut $out, "ok{newln}");
    end match
    return void;
end fn.main
