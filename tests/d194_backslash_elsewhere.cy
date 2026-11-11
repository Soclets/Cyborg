// Only a format print's text says what `\` means.
// trace: D194
import std;

fn.main [none] returns void:
    let $s: str set_to "a\\b";
    return void;
end fn.main
