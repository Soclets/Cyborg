// trace: D159, D158
@config(default);
import std;

fn.main [] -> void:
    let $reading: f32.ieee = 0.5;
    let $clean: f32 = $reading as f32;
    if $clean: f32[min: 0.5, max: 0.5]:
        std::io::fmt_print("clean{newln}");
    end if
    return void;
end fn.main
