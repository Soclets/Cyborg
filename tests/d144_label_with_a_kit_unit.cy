// trace: D144, D112
import std;
@config(default);

fn wait [$t: i32.std::time.seconds] -> void:
    labels [$t as for.std::time.seconds];

    std::io::fmt_print("{}{newln}", ($t as i32));
    return void;
end fn wait

fn.main [] -> void:
    wait(for.std::time.seconds: 2);
    return void;
end fn.main
