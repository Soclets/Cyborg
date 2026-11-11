// It can be handed on, and as with any `struct.must_Give`, nothing can end its duty yet.
// trace: D177, D2
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

fn open [$fd: i32] -> open_file:
    return open_file[fd = $fd];
end fn open

struct.must_Give log:
    file: open_file;
end struct.must_Give log

fn keep [$l: Given log] -> log:
    return Give $l;
end fn keep

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $l: log = keep(log[file = open(3)]);
    let $m: log = Give $l;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $m.file.fd);
    return void;
end fn.main
