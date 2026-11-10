// trace: D180, D2
import std;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    std::io::fmt_print("closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

fn.main [none] -> void:
    let $files: std::list<open_file>[len.max: inf items] = [open_file[fd = 3], open_file[fd = 4]];
    for $f in Give $files:
        if $f.fd == 3:
            close(Give $f);
        end if
    end for
    return void;
end fn.main
