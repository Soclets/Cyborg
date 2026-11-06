// D129 refuses `finish` while a `must_Give` field inside hasn't been given away, but such a
// field is still undecided in 12.8, so this can't be reached yet.
// trace: D129
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

struct.must_Give pair:
    a: open_file;
end struct.must_Give pair

fn.main [] -> void:
    return void;
end fn.main
