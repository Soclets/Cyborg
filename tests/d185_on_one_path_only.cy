// trace: D185, D2
import std;
@config(default);

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
    if $l.lines > 3:
        close(Give $l.file);
    end if
    finish $l;
    return void;
end fn.main
