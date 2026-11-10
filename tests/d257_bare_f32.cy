// The same for `f32`.
// trace: D257
@config(default);
import std;

fn half [$x: f32] -> f32.stable:
    return 0.5;
end fn half

fn.main [] -> void:
    return void;
end fn.main
