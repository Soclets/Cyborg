// trace: D129

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] -> void:
    let $f: open_file = open_file[fd = 3];
    finish $f;
    let $n: i32 = $f.fd;
    return void;
end fn.main
