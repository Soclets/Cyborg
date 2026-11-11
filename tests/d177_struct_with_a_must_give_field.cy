// A struct holding a `must_Give` field says so, and then it's fine.
// trace: D177, D129

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

struct.must_Give pair:
    a: open_file;
end struct.must_Give pair

fn.main [none] returns void:
    return void;
end fn.main
