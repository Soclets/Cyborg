// trace: D86
@config(default);

fn.pure f [] -> i32:
    shield sensor:
        reason "talks to the chip"
        return 1;
    end shield sensor
end fn.pure f

fn.main [] -> void:
    return void;
end fn.main
