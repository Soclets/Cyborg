// trace: D129, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    finish $f;
    return void;
end fn close

fn.main [none] returns void:
    let $f: open_file set_to open_file[fd set_to 3];
    close(Give $f);
    finish $f;
    return void;
end fn.main
