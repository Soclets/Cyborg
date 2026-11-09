// trace: D203, D2
import std;
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [] -> void:
    decl $f: open_file;
    return void;
end fn.main
