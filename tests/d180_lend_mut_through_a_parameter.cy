// trace: D180, D2
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn warm_all [$rs: Lent.mut std::list<reading>[len.max: inf items]] -> void:
    for $r in Lend.mut $rs:
        change $r.celsius = $r.celsius + 5;
    end for
    return void;
end fn warm_all

fn.main [] -> void:
    let mut $readings: std::list<reading>[len.max: inf items] = [reading[celsius = 1]];
    warm_all(Lend.mut $readings);
    for $r in $readings:
        std::println($r.celsius as str);
    end for
    return void;
end fn.main
