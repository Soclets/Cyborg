// trace: D144, D112
@config(default);
import std;

fn wait [$t: i32.std::units::time.seconds.span] -> void:
    labels [$t as for.std::units::time.seconds];

    std::io::fmt_print("{}{newln}", ($t as i32));
    return void;
end fn wait

fn.main [] -> void:
    wait(for.std::units::time.seconds: 2);
    return void;
end fn.main
