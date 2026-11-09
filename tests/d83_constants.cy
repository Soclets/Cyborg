// trace: D83, D59
import std;
@config(default);

const $b: i32 = 60 x 60;

fn.main [] -> void:
    const $c: i32 = $b + 1;
    std::println($b as str);
    std::println($c as str);
    return void;
end fn.main
