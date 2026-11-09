// trace: D134, D2
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

type.new log_file = open_file;

fn f [$l: Given log_file] -> void:
    return void;
end fn f

fn.main [] -> void:
    return void;
end fn.main
