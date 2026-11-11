// trace: D146
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $by: i64 set_to 2;
    let $from: i32 set_to 1;
    let $to: i32 set_to 5;
    for $i in range[from: $from, to: $to, step_by: $by]:
        let $kept: i32 set_to $i;
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $kept);
    end for
    return void;
end fn.main
