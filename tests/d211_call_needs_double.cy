// trace: D211
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn reading.%%warm_up [$r: Lent.mut reading, $by: i32] -> void:
    return void;
end fn reading.%%warm_up

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 20];
    $r.%warm_up(by: 3);
    return void;
end fn.main
