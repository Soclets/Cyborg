// trace: D126, D9, D20
import std;

union.tagged status:
    ok
    failed[code: i32]
    lost[where: i32]
end union.tagged status

fn show [$s: status] -> void:
    if $s: status.failed:
        std::println("failed ", $s.code as str);
    else if $s: status.lost:
        std::println("lost ", $s.where as str);
    else:
        std::println("something else");
    end if
    return void;
end fn show

fn grade [$n: i32] -> void:
    if $n < 0:
        std::println("too low");
    else if $n: i32[min: 0, max: 100]:
        std::println("fits");
    else:
        std::println("too high");
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
