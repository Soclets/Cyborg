// trace: D129, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn sneaky [$f: Lent.mut open_file] -> void:
    finish $f;
    return void;
end fn sneaky

fn.main [] -> void:
    return void;
end fn.main
