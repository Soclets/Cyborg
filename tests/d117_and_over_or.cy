// trace: D117, D49
import std;

fn.pure yes [] -> bool:
    return 1 < 2;
end fn.pure yes

fn.pure no [] -> bool:
    return 2 < 1;
end fn.pure no

fn.main [] -> void:
    if yes() or no() and no():
        std::println("and binds tighter");
    end if
    if not no() and yes():
        std::println("not binds tighter");
    end if
    return void;
end fn.main
