// trace: D128
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    for rows $a in range[from: 1, to: 3]:
        for $b in range[from: 1, to: 3]:
            if $a == 2 and $b == 2:
                break rows;
            end if
            std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $a, $b);
        end for
    end for rows
    std::io::fmt_print(Lend.mut $out, "done{newln}");
    return void;
end fn.main
