// trace: D129, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    let $f: open_file set_to open_file[fd set_to 3];
    if 1 below 2:
        finish $f;
    end if
    return void;
end fn.main
