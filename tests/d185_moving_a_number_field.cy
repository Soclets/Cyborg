// A number is copied, and takes no word.
// trace: D185, D123
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

fn.main [none] returns void:
    let $l: log set_to log[file set_to open_file[fd set_to 4], lines set_to 7];
    let $n: i32 set_to Give $l.lines;
    return void;
end fn.main
