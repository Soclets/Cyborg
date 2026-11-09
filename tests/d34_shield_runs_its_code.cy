// A shield says it's dangerous, and what's in it still runs.
// trace: D34
import std;
@config(default);

fn.main [] -> void:
    shield sensor:
        reason "talks to the chip"
        std::println("inside");
    end shield sensor
    return void;
end fn.main
