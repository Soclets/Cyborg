// trace: D159, D158
import std;

fn.main [] -> void:
    let $reading: f64.ieee = 1.5;
    let $clean: f64 = $reading as f64;
    if $clean: f64[min: 1.5, max: 1.5]:
        std::println("clean");
    end if
    return void;
end fn.main
