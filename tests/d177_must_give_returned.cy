// trace: D177, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn open [none] returns open_file:
    return open_file[fd set_to 3];
end fn open

fn.main [none] returns void:
    return void;
end fn.main
