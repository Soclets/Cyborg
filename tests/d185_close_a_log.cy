// The standard's `close_log`: a `struct.must_Give` ends its duty at last.
// trace: D185, D177, D129
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

struct.must_Give log:
    file: open_file;
    lines: i32;
end struct.must_Give log

fn close_log [$l: Given log] returns void:
    close(Give $l.file);
    finish $l;
    return void;
end fn close_log

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $l: log set_to log[file set_to open_file[fd set_to 3], lines set_to 1];
    close_log(Give $l);
    std::io::fmt_print(Lend.mut $out, "done{newln}");
    return void;
end fn.main
