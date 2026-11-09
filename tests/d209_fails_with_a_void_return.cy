// trace: D209
@config(default);
import std;

fn check [$n: i32] -> void fails [negative]:
    if $n < 0:
        fail negative;
    end if
    return void;
end fn check

fn.main [] -> void:
    check(-1) otherwise:
        case negative:
            std::io::fmt_print("negative{newln}");
    end otherwise
    return void;
end fn.main
