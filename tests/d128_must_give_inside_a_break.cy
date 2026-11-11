// trace: D128, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] returns void:
    forever:
        let $f: open_file set_to open_file[fd set_to 3];
        break;
    end forever
    return void;
end fn.main
