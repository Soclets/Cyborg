// trace: D128, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [] -> void:
    forever:
        let $f: open_file = open_file[fd = 3];
        break;
    end forever
    return void;
end fn.main
