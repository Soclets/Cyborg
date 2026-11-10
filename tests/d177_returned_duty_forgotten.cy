// trace: D177, D2
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

fn.main [none] -> void:
    let $f: open_file = open(3);
    return void;
end fn.main
