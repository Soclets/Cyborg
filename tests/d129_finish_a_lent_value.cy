// The caller still holds the duty.
// trace: D129, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn sneaky [$f: open_file] -> void:
    finish $f;
    return void;
end fn sneaky

fn.main [] -> void:
    return void;
end fn.main
