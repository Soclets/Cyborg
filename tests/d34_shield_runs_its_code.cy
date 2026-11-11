// A shield says it's dangerous, and what's in it still runs.
// trace: D34
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    shield sensor:
        reason "talks to the chip"
        std::io::fmt_print(Lend.mut $out, "inside{newln}");
    end shield sensor
    return void;
end fn.main
