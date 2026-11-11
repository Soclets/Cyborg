// trace: D180, D2
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

fn first [$files: Given std::list<open_file>[len.max: inf items]] returns i32:
    for $f in Give $files:
        let $n: i32 set_to $f.fd;
        close(Give $f);
        return $n;
    end for
    return 0;
end fn first

fn.main [none] returns void:
    let $files: std::list<open_file>[len.max: inf items] set_to [open_file[fd set_to 3], open_file[fd set_to 4]];
    let $n: i32 set_to first(Give $files);
    return void;
end fn.main
