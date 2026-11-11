// trace: D177, D180, D129
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

fn open [$fd: i32] returns open_file:
    return open_file[fd set_to $fd];
end fn open

fn.main [none] returns void:
    let $files: std::list<open_file>[len.max: inf items] set_to [open(3)];
    finish $files;
    return void;
end fn.main
