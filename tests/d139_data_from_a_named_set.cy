// trace: D139, D47
import std;
@config(default);

fam.failure file_failed:
    file_failed.not_found;
    file_failed.disk_full[free: i32, label: str];
end fam.failure file_failed

fn save [] -> void:
    fails [file_failed];

    fail file_failed.disk_full[free = 7, label = "scratch"];
end fn save

fn.main [] -> void:
    save() otherwise:
        case not_found:
            std::io::fmt_print("not found{newln}");
        case file_failed.disk_full $d:
            std::io::fmt_print("{} has {} free{newln}", $d.label, $d.free);
    end otherwise
    return void;
end fn.main
