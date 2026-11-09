// Only a format print's text says what `\` means.
// trace: D194
import std;
@config(default);

fn.main [] -> void:
    let $s: str = "a\\b";
    return void;
end fn.main
