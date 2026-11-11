// trace: D126, D9, D20
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
    lost[where: i32];
end union.tagged status

fn show [$s: status] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if $s: status.failed:
        std::io::fmt_print(Lend.mut $out, "failed {}{newln}", $s.code);
    else if $s: status.lost:
        std::io::fmt_print(Lend.mut $out, "lost {}{newln}", $s.where);
    else:
        std::io::fmt_print(Lend.mut $out, "something else{newln}");
    end if
    return void;
end fn show

fn grade [$n: i32] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if $n below 0:
        std::io::fmt_print(Lend.mut $out, "too low{newln}");
    else if $n: i32[min: 0, max: 100]:
        std::io::fmt_print(Lend.mut $out, "fits{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "too high{newln}");
    end if
    return void;
end fn grade

fn.main [none] -> void:
    show(status.failed[code = 1]);
    show(status.lost[where = 2]);
    show(status.ok);
    grade(-1);
    grade(50);
    grade(101);
    return void;
end fn.main
