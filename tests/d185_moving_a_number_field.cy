// A number is copied, and takes no word.
// trace: D185, D123
@config(default);
import std;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    std::io::fmt_print("closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

struct.must_Give log:
    file: open_file;
    lines: i32;
end struct.must_Give log

fn.main [] -> void:
    let $l: log = log[file = open_file[fd = 4], lines = 7];
    let $n: i32 = Give $l.lines;
    return void;
end fn.main
