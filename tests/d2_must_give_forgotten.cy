// trace: D2, D57
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    return void;
end fn.main
