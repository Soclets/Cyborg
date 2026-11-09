// trace: D117, D49
@config(default);
import std;

fn.pure yes [] -> bool:
    return 1 < 2;
end fn.pure yes

fn.pure no [] -> bool:
    return 2 < 1;
end fn.pure no

fn.main [] -> void:
    if yes() or no() and no():
        std::io::fmt_print("and binds tighter{newln}");
    end if
    if not no() and yes():
        std::io::fmt_print("not binds tighter{newln}");
    end if
    return void;
end fn.main
