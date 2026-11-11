// A skipped decl still creates its variable, with the type it wrote, so `$x.fd` is an `i32`.
// trace: D215, D203
import std;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    decl $x: open_file;
    assign $x = open_file[fd = 3];
    let $n: i64 = $x.fd;
    finish $x;
    return void;
end fn.main
