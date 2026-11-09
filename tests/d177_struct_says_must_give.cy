// Building and lending one works, but nothing can end its duty yet: `finish` waits for the file,
// and giving one field away is still open.
// trace: D177, D129
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

fn open [$fd: i32] -> open_file:
    return open_file[fd = $fd];
end fn open

struct.must_Give log:
    file: open_file;
    lines: i32;
end struct.must_Give log

fn read_log [$l: log] -> i32:
    return $l.file.fd + $l.lines;
end fn read_log

fn.main [] -> void:
    let $f: open_file = open(3);
    let $l: log = log[file = Give $f, lines = 10];
    std::io::fmt_print("{}{newln}", read_log(Lend $l));
    return void;
end fn.main
