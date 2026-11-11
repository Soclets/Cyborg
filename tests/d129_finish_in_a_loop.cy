// trace: D129, D128

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    let $f: open_file = open_file[fd = 3];
    for $i in range[from: 1, to: 2]:
        finish $f;
    end for
    return void;
end fn.main
