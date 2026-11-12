// A field holding text that isn't a literal stores owned text instead.
// trace: D367
import std;

struct room:
    name: str;
end struct room

fn named [$text: str] -> room:
    return room[name = $text];
end fn named

fn.main [none] -> void:
    return void;
end fn.main
