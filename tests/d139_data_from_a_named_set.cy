// trace: D139, D47
import std;
import os;

fam.failure file_failed:
    file_failed.not_found;
    file_failed.disk_full[free: i32, label: str];
end fam.failure file_failed

fn save [none] -> void fails [file_failed]:
    fail file_failed.disk_full[free = 7, label = "scratch"];
end fn save

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    save() otherwise:
        case not_found:
            std::io::fmt_print(Lend.mut $out, "not found{newln}");
        case file_failed.disk_full $d:
            std::io::fmt_print(Lend.mut $out, "{} has {} free{newln}", $d.label, $d.free);
    end otherwise
    return void;
end fn.main
