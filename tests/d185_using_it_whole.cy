// trace: D185
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

struct.must_Give log:
    file: open_file;
    lines: i32;
end struct.must_Give log

fn keep [$l: Given log] returns void:
    close(Give $l.file);
    finish $l;
    return void;
end fn keep

fn.main [none] returns void:
    let $l: log = log[file = open_file[fd = 4], lines = 7];
    close(Give $l.file);
    keep(Give $l);
    return void;
end fn.main
