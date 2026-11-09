// trace: D180, D2
@config(default);
import std;

fn make [] -> std::list<i32>[len.max: inf items]:
    return [1, 2];
end fn make

fn.main [] -> void:
    for $n in Give make():
        return void;
    end for
    return void;
end fn.main
