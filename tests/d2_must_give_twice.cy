// A definite refusal wins over how the receiving function's duty ends, which isn't decided.
// trace: D2, D57
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    return void;
end fn close

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    close(Give $f);
    close(Give $f);
    return void;
end fn.main
