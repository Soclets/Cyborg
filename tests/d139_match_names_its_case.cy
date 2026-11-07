// trace: D139, D9
import std;

union.tagged status:
    ok
    failed[code: i32]
    lost[where: str]
end union.tagged status

fn show [$s: status] -> void:
    match $s:
        case status.failed $f:
            std::println("failed ", $f.code as str);
        case status.lost $l:
            std::println("lost at ", Lend $l.where);
        case status.ok:
            std::println("ok");
    end match
    return void;
end fn show

fn make [] -> status:
    return status.failed[code = 8];
end fn make

fn.main [] -> void:
    show(status.failed[code = 7]);
    show(status.lost[where = "home"]);
    show(status.ok);
    match make():
        case status.failed $f:
            std::println("fresh ", $f.code as str);
        case status.lost:
            std::println("lost");
        case status.ok:
            std::println("ok");
    end match
    return void;
end fn.main
