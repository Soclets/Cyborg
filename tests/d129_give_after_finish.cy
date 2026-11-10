// `finish` ends the duty, and nothing says the value itself is dead.
// trace: D129

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    finish $f;
    return void;
end fn close

fn.main [none] -> void:
    let $f: open_file = open_file[fd = 3];
    finish $f;
    close(Give $f);
    return void;
end fn.main
