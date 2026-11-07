// trace: D149
import std;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn take [$w: Given wrapped] -> void:
    let $r: reading = Give $w as reading;
    std::println($r.celsius as str);
    return void;
end fn take

fn make [$r: Given reading] -> wrapped:
    return Give $r as wrapped;
end fn make

fn.main [] -> void:
    let $a: reading = reading[celsius = 1];
    let $b: reading = reading[celsius = 2];
    take(Give $a as wrapped);
    take(make(Copy $b));
    return void;
end fn.main
