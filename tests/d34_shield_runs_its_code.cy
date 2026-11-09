// A shield says it's dangerous, and what's in it still runs.
// trace: D34
@config(default);
import std;

fn.main [] -> void:
    shield sensor:
        reason "talks to the chip"
        std::io::fmt_print("inside{newln}");
    end shield sensor
    return void;
end fn.main
