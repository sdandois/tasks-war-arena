use std::io::{Read, Write};

use apache_avro::{
    types::{Record, Value},
    Schema, Writer,
};

use anyhow::Result;

use super::*;

mod schemas {
    pub const TASK_ID: &'static str = r#"
{
        "name": "TaskId",
        "type": "record",
        "fields": [
            {
                "name": "player_id",
                "type": "int"
            },
            {
                "name": "number",
                "type": "int"
            }
        ]
    }
"#;

pub const KILL_EVENT: &'static str = r#"
{
    "type": "record",
    "name": "KillEvent",
    "fields": [
        {
            "name": "reason",
            "type": "string"
        },
        {
            "name": "task_id",
            "type": "TaskId",
        }
    ]
}
"#;


}

fn write_history_event<W: Write>(h: &HistoryEntry, w: W) -> Result<()> {
    let schema = Schema::parse_list(&[schemas::TASK_ID, schemas::KILL_EVENT]).unwrap();

    let mut w = Writer::new(&schema[1], w);

    let mut t = Record::new(&schema[0]).unwrap();

    t.put("player", 0);
    t.put("num", 1);

    
    let mut r = Record::new(&schema[1]).unwrap();
    
    r.put("reason", "Out of bad luck".to_string());
    r.put("task_id", t);
    
    w.append(r)?;
    Ok(())
}

fn read_history_event<R: Read>(r: R) -> Result<HistoryEntry> {
    Ok(HistoryEntry::KillEvent(TaskId(0, 0), " dfa".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avro_serialize_kill_event() {
        let history_event: HistoryEntry =
            HistoryEntry::KillEvent(TaskId(0, 1), String::from("out of bad luck"));

        let mut v = Vec::<u8>::new();

        write_history_event(&history_event, &mut v).unwrap();

        let h: HistoryEntry = read_history_event(v.as_slice()).unwrap();

        assert_eq!(history_event, h);
    }
}
