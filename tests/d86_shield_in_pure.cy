// trace: D86

fn.pure f [none] returns i32:
    shield sensor:
        reason "talks to the chip"
        return 1;
    end shield sensor
end fn.pure f

fn.main [none] returns void:
    return void;
end fn.main
