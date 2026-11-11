// trace: D2, D57

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    finish $f;
    return void;
end fn close

fn.main [none] returns void:
    let $f: open_file = open_file[fd = 3];
    close(Give $f);
    close(Give $f);
    return void;
end fn.main
