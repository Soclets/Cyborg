// trace: D177, D2
import std;

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

fn steal [$f: open_file] -> open_file:
    return Give $f;
end fn steal

fn.main [] -> void:
    return void;
end fn.main
