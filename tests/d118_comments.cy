// A note for whoever reads this.
// trace: D118, D119
import std;
import os;

/// Says hello.
fn hello [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    // a note inside
    std::io::fmt_print(Lend.mut $out, "hello{newln}"); // and one after code
    //
    return void;
end fn hello

comment why_this_exists:
    This text isn't code: let $x: i32 = "nothing";
    comment nested_one:
        Nor is this.
    end comment nested_one
    comment:is not an opener, because it isn't alone like that
end comment why_this_exists

fn.main [none] -> void:
    hello();
    return void;
end fn.main
