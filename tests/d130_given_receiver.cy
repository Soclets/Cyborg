// trace: D130, D8

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn open_file.%close [$f: Given open_file] returns void:
    finish $f;
    return void;
end fn open_file.%close

fn.main [none] returns void:
    return void;
end fn.main
