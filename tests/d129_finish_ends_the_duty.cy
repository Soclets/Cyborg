// A whole `must_Give` program: made, lent, handed on, and finished.
// trace: D129, D2, D57
import std;
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn read_from [$f: open_file] -> i32:
    return $f.fd + 1;
end fn read_from

fn pass_on [$f: Given open_file] -> void:
    close(Give $f);
    return void;
end fn pass_on

fn close [$f: Given open_file] -> void:
    std::println("closing ", $f.fd as str);
    finish $f;
    return void;
end fn close

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    std::println(read_from(Lend $f) as str);
    pass_on(Give $f);
    let $g: open_file = open_file[fd = 4];
    finish $g;
    std::println("done");
    return void;
end fn.main
