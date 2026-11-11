// trace: D129

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    let $f: open_file set_to open_file[fd set_to 3];
    finish $f;
    let $n: i32 set_to $f.fd;
    return void;
end fn.main
