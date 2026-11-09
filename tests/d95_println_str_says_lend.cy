// A `str` in a variable is passed with a word, like anything that isn't a plain number.
// trace: D95, D3
import std;
@config(default);

fn.main [] -> void:
    let $s: str = "hi";
    std::println($s);
    return void;
end fn.main
