// trace: D177, D2
import std;
import os;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "closing {}{newln}", $f.fd);
    finish $f;
    return void;
end fn close

fn open [$fd: i32] returns open_file:
    return open_file[fd set_to $fd];
end fn open

fn pass [$f: Given open_file] returns open_file:
    return Give $f;
end fn pass

fn.main [none] returns void:
    close(pass(open(5)));
    return void;
end fn.main
