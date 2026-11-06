// Every way out of this `forever` gave it away, so `main` has done its part, and what's
// left is how `close` ends the duty.
// trace: D128, D2
struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    return void;
end fn close

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    forever:
        close(Give $f);
        break;
    end forever
    return void;
end fn.main
