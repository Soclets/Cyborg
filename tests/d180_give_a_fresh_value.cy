// trace: D180, D2
import std;

fn make [none] returns std::list<i32>[len.max: inf items]:
    return [1, 2];
end fn make

fn.main [none] returns void:
    for $n in Give make():
        return void;
    end for
    return void;
end fn.main
