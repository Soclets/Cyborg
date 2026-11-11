// Being given a duty and doing nothing with it is forgetting it.
// trace: D129, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    return void;
end fn close

fn.main [none] returns void:
    return void;
end fn.main
