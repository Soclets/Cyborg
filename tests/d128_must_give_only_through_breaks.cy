// Every way out of this `forever` gave it away.
// trace: D128, D2

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    finish $f;
    return void;
end fn close

fn.main [none] -> void:
    let $f: open_file = open_file[fd = 3];
    forever:
        close(Give $f);
        break;
    end forever
    return void;
end fn.main
