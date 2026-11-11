// With no `else:`, the chain can run no block at all.
// trace: D126, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    finish $f;
    return void;
end fn close

fn.main [none] -> void:
    let $f: open_file = open_file[fd = 3];
    if 1 below 2:
        close(Give $f);
    else if 2 below 1:
        close(Give $f);
    end if
    return void;
end fn.main
