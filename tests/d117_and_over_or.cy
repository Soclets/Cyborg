// trace: D117, D49
import std;

fn.pure yes [none] -> bool:
    return 1 < 2;
end fn.pure yes

fn.pure no [none] -> bool:
    return 2 < 1;
end fn.pure no

fn.main [none] -> void:
    if yes() or no() and no():
        std::io::fmt_print("and binds tighter{newln}");
    end if
    if not no() and yes():
        std::io::fmt_print("not binds tighter{newln}");
    end if
    return void;
end fn.main
