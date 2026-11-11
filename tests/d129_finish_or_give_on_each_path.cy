// trace: D129, D2, D126
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    finish $f;
    return void;
end fn close

fn end_it [$f: Given open_file, $quick: bool] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    if $quick:
        finish $f;
    else:
        close(Give $f);
    end if
    std::io::fmt_print(Lend.mut $out, "ended{newln}");
    return void;
end fn end_it

fn.main [none] returns void:
    end_it(open_file[fd set_to 1], true);
    end_it(open_file[fd set_to 2], false);
    return void;
end fn.main
