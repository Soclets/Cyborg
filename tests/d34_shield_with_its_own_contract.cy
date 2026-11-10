// How a shield's own entry and exit are written isn't decided.
// trace: D37, D39

fn.main [none] -> void:
    shield sensor:
        reason "talks to the chip"
        requires 1 < 2;
    end shield sensor
    return void;
end fn.main
