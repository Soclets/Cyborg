// trace: D205, D119
@config(default); // the compiler's defaults
import std; // for the print

fn.main [] -> void: // where it starts
    let $x: i32 = 1; // a note
    if $x > 0: // always
        std::io::fmt_print("{}{newln}", $x); // prints 1
    end if // the if is done
    std::io::fmt_print("a // b{newln}"); // the // inside the text is text
    return void;
end fn.main // the end
