// trace: D126, D9, D20
@config(default);
import std;

union.tagged status:
    ok;
    failed[code: i32];
    lost[where: i32];
end union.tagged status

fn show [$s: status] -> void:
    if $s: status.failed:
        std::io::fmt_print("failed {}{newln}", $s.code);
    else if $s: status.lost:
        std::io::fmt_print("lost {}{newln}", $s.where);
    else:
        std::io::fmt_print("something else{newln}");
    end if
    return void;
end fn show

fn grade [$n: i32] -> void:
    if $n < 0:
        std::io::fmt_print("too low{newln}");
    else if $n: i32[min: 0, max: 100]:
        std::io::fmt_print("fits{newln}");
    else:
        std::io::fmt_print("too high{newln}");
    end if
    return void;
end fn grade

fn.main [] -> void:
    show(status.failed[code = 1]);
    show(status.lost[where = 2]);
    show(status.ok);
    grade(-1);
    grade(50);
    grade(101);
    return void;
end fn.main
