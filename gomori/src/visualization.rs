use crate::Field;

pub fn visualize_top_cards(fields: &[Field]) -> String {
    let (mut i_min, mut i_max, mut j_min, mut j_max) =
        (fields[0].i, fields[0].i, fields[0].j, fields[0].j);
    for field in fields {
        i_min = i_min.min(field.i);
        i_max = i_max.max(field.i);
        j_min = j_min.min(field.j);
        j_max = j_max.max(field.j);
    }

    let mut cursor_i = i_min;
    let mut cursor_j = j_min;
    // Draw the top of the box
    let mut result = format!("    {:>2}", j_min);
    result += "\n    ╭";
    for _ in j_min..=j_max {
        result += "──";
    }
    result += &format!("╮\n{:>3} │", i_min);

    for field in fields {
        while cursor_i < field.i {
            while cursor_j <= j_max {
                result += "  ";
                cursor_j += 1;
            }
            cursor_i += 1;
            result += &format!("│\n{:>3} │", cursor_i);
            cursor_j = j_min;
        }
        while cursor_j < field.j {
            result += "  ";
            cursor_j += 1;
        }
        if let Some(card) = field.top_card {
            result += &format!("{} ", card);
        } else if !field.hidden_cards.is_empty() {
            result += "🂠 ";
        } else {
            result += "  ";
        }
        cursor_j += 1;
    }
    // Draw the bottom of the box
    while cursor_j <= j_max {
        result += "  ";
        cursor_j += 1;
    }
    result += "│\n    ╰";
    for _ in j_min..=j_max {
        result += "──";
    }
    result += "╯";
    result
}
