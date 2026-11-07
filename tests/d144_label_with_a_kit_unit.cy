// trace: D144, D112
import std;

fn wait [$t: i32.std::time.seconds] -> void:
    labels [$t as for.std::time.seconds];

    std::println(($t as i32) as str);
    return void;
end fn wait

fn.main [] -> void:
    wait(for.std::time.seconds: 2);
    return void;
end fn.main
