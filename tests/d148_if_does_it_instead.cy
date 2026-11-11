// trace: D148, D20
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $start: i32 set_to 150;
    if $start: i32[min: 100, max: 200]:
        for $i in range[from: $start, to: 151]:
            std::io::fmt_print(Lend.mut $out, "{}{newln}", $i);
        end for
    end if
    return void;
end fn.main
