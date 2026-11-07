// trace: D149, D2
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn make [] -> reading:
    return reading[celsius = 3];
end fn make

fn.main [] -> void:
    let $w: wrapped = make() as wrapped;
    let $v: wrapped = reading[celsius = 4] as wrapped;
    std::println("ok");
    return void;
end fn.main
