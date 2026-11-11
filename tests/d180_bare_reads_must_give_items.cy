// trace: D180, D182
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $files: std::list<open_file>[len.max: inf items] = [open_file[fd = 3], open_file[fd = 4]];
    for $f in $files:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $f.fd);
    end for
    for $f in Give $files:
        close(Give $f);
    end for
    return void;
end fn.main
