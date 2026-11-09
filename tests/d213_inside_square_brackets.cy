// No space follows an opening `[`.
// trace: D213
@config(default);
import std;

struct reading:
    celsius: i32;
end struct reading

fn.main [] -> void:
    let $r: reading = reading[ celsius = 1];
    return void;
end fn.main
