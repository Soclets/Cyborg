// trace: D130, D8
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn open_file.%close [$f: Given open_file] -> void:
    finish $f;
    return void;
end fn open_file.%close

fn.main [] -> void:
    return void;
end fn.main
