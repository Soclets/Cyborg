// trace: D180, D2
import std;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    std::println("closing ", $f.fd as str);
    finish $f;
    return void;
end fn close

fn first [$files: Given std::list<open_file>[len.max: inf items]] -> i32:
    for $f in Give $files:
        let $n: i32 = $f.fd;
        close(Give $f);
        return $n;
    end for
    return 0;
end fn first

fn.main [] -> void:
    let $files: std::list<open_file>[len.max: inf items] = [open_file[fd = 3], open_file[fd = 4]];
    let $n: i32 = first(Give $files);
    return void;
end fn.main
