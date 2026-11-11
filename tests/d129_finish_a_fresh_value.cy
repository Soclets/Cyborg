// trace: D129

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    finish open_file[fd set_to 3];
    return void;
end fn.main
