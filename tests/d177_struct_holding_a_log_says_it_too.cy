// trace: D177, D3
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

struct archive:
    current: log;
end struct archive

fn.main [] -> void:
    return void;
end fn.main
