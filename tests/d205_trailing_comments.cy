// trace: D205, D119
import std; // for the print
import os;

fn.main [none] returns void: // where it starts
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: i32 = 1; // a note
    if $x above 0: // always
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $x); // prints 1
    end if // the if is done
    std::io::fmt_print(Lend.mut $out, "a // b{newln}"); // the // inside the text is text
    return void;
end fn.main // the end
