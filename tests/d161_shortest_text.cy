// trace: D161
import std;

fn show [$x: f64] -> void:
    std::println($x as str);
    return void;
end fn show

fn.main [] -> void:
    show(0.5);
    show(1.0);
    show(-1.5);
    show(100.0);
    show(~0.3);
    show(~3.14159265358979323846);
    show(~1.7976931348623157);
    return void;
end fn.main
