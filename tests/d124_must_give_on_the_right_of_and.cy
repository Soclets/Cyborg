// The right side only runs if it's needed, so this may never give `$f` away.
// trace: D124, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> bool:
    return 1 < 2;
end fn close

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    if 2 < 1 and close(Give $f):
        return void;
    end if
    return void;
end fn.main
