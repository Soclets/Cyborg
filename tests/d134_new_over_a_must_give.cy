// trace: D134, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

type.new log_file set_to open_file;

fn f [$l: Given log_file] returns void:
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
