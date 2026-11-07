// trace: D164, D162
import std;

fn.main [] -> void:
    let $one: f64 = 1.0;
    let $third: f64 = $one div 3.0;
    std::println($third as str);
    std::println.stored($third);
    let $small: f32 = 1.0;
    let $f: f32 = $small div 3.0;
    std::println($f as str);
    std::println.stored($f);
    return void;
end fn.main
