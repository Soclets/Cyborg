// Returning a view of another string as a bare `str` hides where it's from, so it's refused.
// trace: D367, D170
import std;

fn first [$text: str] -> str:
    return $text;
end fn first

fn.main [none] -> void:
    return void;
end fn.main
