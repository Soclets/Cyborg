// How the duty ends in the function that receives it isn't decided.
// trace: D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    return void;
end fn close

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    close(Give $f);
    return void;
end fn.main
