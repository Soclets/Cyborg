// Each way out of the loop ended the duty.
// trace: D129, D128
@config(default);
import std;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [] -> void:
    let $f: open_file = open_file[fd = 3];
    forever:
        finish $f;
        break;
    end forever
    std::io::fmt_print("out{newln}");
    return void;
end fn.main
