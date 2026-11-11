// D177 is about structs, so a union case holding one is still open.
// trace: D177
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

fn open [$fd: i32] -> open_file:
    return open_file[fd = $fd];
end fn open

union.tagged maybe_file:
    some[file: open_file];
    none;
end union.tagged maybe_file

fn.main [none] -> void:
    return void;
end fn.main
