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

fn close_all [$files: std::list<open_file>[len.max: inf items]] -> void:
    for $f in Give $files:
        close(Give $f);
    end for
    return void;
end fn close_all

fn.main [] -> void:
    return void;
end fn.main
