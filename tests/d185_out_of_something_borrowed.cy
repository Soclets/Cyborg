// trace: D185
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

fn peek [$l: log] -> void:
    close(Give $l.file);
    return void;
end fn peek

fn.main [none] -> void:
    return void;
end fn.main
