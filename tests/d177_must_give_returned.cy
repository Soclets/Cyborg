// trace: D177, D2
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn open [] -> open_file:
    return open_file[fd = 3];
end fn open

fn.main [] -> void:
    return void;
end fn.main
