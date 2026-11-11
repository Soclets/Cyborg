// A skipped decl still creates its variable, with the type it wrote, so `$x.fd` is an `i32`.
// trace: D215, D203
import std;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    decl $x: open_file;
    assign $x set_to open_file[fd set_to 3];
    let $n: i64 set_to $x.fd;
    finish $x;
    return void;
end fn.main
