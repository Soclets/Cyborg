// `main` has done its part here, so all that's left is how `close` ends the duty.
// trace: D126, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    return void;
end fn close

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    if 1 < 2:
        close(Give $f);
    else if 2 < 1:
        close(Give $f);
    else:
        close(Give $f);
    end if
    return void;
end fn.main
