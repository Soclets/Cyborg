// A chain stops at its first false, so this may never run.
// trace: D124, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> i32:
    finish $f;
    return 3;
end fn close

fn.main [none] -> void:
    let $f: open_file = open_file[fd = 3];
    if 2 below 1 below close(Give $f):
        return void;
    end if
    return void;
end fn.main
