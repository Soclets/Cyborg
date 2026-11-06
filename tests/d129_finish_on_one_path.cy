// trace: D129, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    if 1 < 2:
        finish $f;
    end if
    return void;
end fn.main
