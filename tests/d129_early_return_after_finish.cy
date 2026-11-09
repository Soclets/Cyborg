// trace: D129, D2
import std;
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn f [$quick: bool] -> void:
    let $g: open_file = open_file[fd = 3];
    if $quick:
        finish $g;
        return void;
    end if
    std::io::fmt_print("slow{newln}");
    finish $g;
    return void;
end fn f

fn.main [] -> void:
    f(true);
    f(false);
    return void;
end fn.main
