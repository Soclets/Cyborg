// trace: D211
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn reading.%warm_up [$r: Lent.mut reading] -> void:
    return void;
end fn reading.%warm_up

fn.main [] -> void:
    return void;
end fn.main
