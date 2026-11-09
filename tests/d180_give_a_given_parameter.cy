// trace: D180, D2
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

fn close_all [$files: Given std::list<open_file>[len.max: inf items]] -> void:
    for $f in Give $files:
        close(Give $f);
    end for
    return void;
end fn close_all

fn.main [] -> void:
    let $files: std::list<open_file>[len.max: inf items] = [open_file[fd = 3], open_file[fd = 4]];
    close_all(Give $files);
    return void;
end fn.main
