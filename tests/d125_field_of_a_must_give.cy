// Changing a field isn't handing it off, so the duty is still there.
// trace: D125, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    let mut $f: open_file = open_file[fd = 3];
    change $f.fd = 4;
    return void;
end fn.main
