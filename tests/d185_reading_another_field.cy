// "The only things left" are moving out another field, or `finish`, so even this read waits.
// trace: D185
import std;
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    std::println("closing ", $f.fd as str);
    finish $f;
    return void;
end fn close

struct.must_Give log:
    file: open_file;
    lines: i32;
end struct.must_Give log

fn.main [] -> void:
    let $l: log = log[file = open_file[fd = 4], lines = 7];
    close(Give $l.file);
    std::println($l.lines as str);
    finish $l;
    return void;
end fn.main
