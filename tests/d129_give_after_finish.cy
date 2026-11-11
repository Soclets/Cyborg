// `finish` ends the duty, and nothing says the value itself is dead.
// trace: D129

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    finish $f;
    return void;
end fn close

fn.main [none] returns void:
    let $f: open_file set_to open_file[fd set_to 3];
    finish $f;
    close(Give $f);
    return void;
end fn.main
