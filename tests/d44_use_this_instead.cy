// trace: D44
import std;
import os;

fn read_port [$ok: bool] returns i32 fails [no_port]:
    if $ok:
        return 9000;
    end if
    fail no_port;
end fn read_port

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to read_port(1 below 2) ?? 8080;
    let $b: i32 set_to read_port(2 below 1) ?? 8080;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
