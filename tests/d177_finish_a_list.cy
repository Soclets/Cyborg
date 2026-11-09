// trace: D177, D180, D129
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

fn open [$fd: i32] -> open_file:
    return open_file[fd = $fd];
end fn open

fn.main [] -> void:
    let $files: std::list<open_file>[len.max: inf items] = [open(3)];
    finish $files;
    return void;
end fn.main
