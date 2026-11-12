// trace: D180, D129
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $heap: os::heap = os::heap();
    let $files: std::coll::list<open_file>[len.max: inf items] = std::coll::list.from([open_file[fd = 3], open_file[fd = 4]], alloc: Lend $heap);
    for $f in Give $files:
        close(Give $f);
    end for
    std::io::fmt_print(Lend.mut $out, "all closed{newln}");
    return void;
end fn.main
