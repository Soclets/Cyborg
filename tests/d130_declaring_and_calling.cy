// trace: D130, D4, D5, D8
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn reading.%warm_up [$r: Lent.mut reading, $by: i32] -> void:
    labels [$by as by];
    change $r.celsius = $r.celsius + $by;
    return void;
end fn reading.%warm_up

fn reading.%show [$r: reading] -> void:
    std::println($r.celsius as str);
    return void;
end fn reading.%show

fn.pure reading.%doubled [$r: reading] -> i32:
    return $r.celsius x 2;
end fn.pure reading.%doubled

fn make [] -> reading:
    return reading[celsius = 5];
end fn make

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21];
    $r.%warm_up(by: 2);
    $r.%show();
    $r.%warm_up(by: 1);
    $r.%show();
    std::println($r.%doubled() as str);
    make().%show();
    return void;
end fn.main
