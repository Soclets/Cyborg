// trace: D177, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

struct holder:
    f: open_file;
end struct holder

fn.main [none] returns void:
    return void;
end fn.main
