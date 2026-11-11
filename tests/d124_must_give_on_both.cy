// A narrowing runs both sides, so this one always gives it away.
// trace: D124, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns bool:
    finish $f;
    return 1 below 2;
end fn close

fn.main [none] returns void:
    let $f: open_file set_to open_file[fd set_to 3];
    if 2 below 1 and.both close(Give $f):
        return void;
    end if
    return void;
end fn.main
