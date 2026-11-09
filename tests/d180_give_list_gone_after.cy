// trace: D180, D2
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

fn.main [] -> void:
    let $files: std::list<open_file>[len.max: inf items] = [open_file[fd = 3], open_file[fd = 4]];
    for $f in Give $files:
        close(Give $f);
    end for
    std::io::fmt_print("{}{newln}", $files.len);
    return void;
end fn.main
