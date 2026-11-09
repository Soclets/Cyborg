// It can be handed on, and as with any `struct.must_Give`, nothing can end its duty yet.
// trace: D177, D2
import std;
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    std::println("closing ", $f.fd as str);
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

fn.main [] -> void:
    let $l: log = keep(log[file = open(3)]);
    let $m: log = Give $l;
    std::println($m.file.fd as str);
    return void;
end fn.main
