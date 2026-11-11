// trace: D129

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] -> void:
    finish open_file[fd = 3];
    return void;
end fn.main
