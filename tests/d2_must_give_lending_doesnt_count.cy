// trace: D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn peek [$f: open_file] returns i32:
    return $f.fd;
end fn peek

fn.main [none] returns void:
    let $f: open_file set_to open_file[fd set_to 3];
    let $n: i32 set_to peek(Lend $f);
    return void;
end fn.main
