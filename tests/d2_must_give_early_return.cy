// trace: D2
@config(default);

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    finish $f;
    return void;
end fn close

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    if 1 < 2:
        return void;
    end if
    close(Give $f);
    return void;
end fn.main
