// trace: D129, D2
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn f [$quick: bool] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $g: open_file = open_file[fd = 3];
    if $quick:
        finish $g;
        return void;
    end if
    std::io::fmt_print(Lend.mut $out, "slow{newln}");
    finish $g;
    return void;
end fn f

fn.main [none] returns void:
    f(true);
    f(false);
    return void;
end fn.main
