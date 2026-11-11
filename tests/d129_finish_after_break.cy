// Each way out of the loop ended the duty.
// trace: D129, D128
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $f: open_file = open_file[fd = 3];
    forever:
        finish $f;
        break;
    end forever
    std::io::fmt_print(Lend.mut $out, "out{newln}");
    return void;
end fn.main
