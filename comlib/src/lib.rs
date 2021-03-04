use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/*
    waka time json schema

    summaries_detail := {
        digital: string
        hours: i32
        minutes: i32
        name: Option<string>
        percent: f32
        seconds: i32
        text: string
        total_seconds: f32
        machine_name_id: Option<string>     // exists machines only
    }

    sumaries_data := {
        branches: Option<[ <summaries_detail> ]>    // exists project_summaries only
        entities: Option<[ <summaries_detail> ]>    // exists project_summaries only
        categories: [ <summaries_detail> ]
        dependencies: [ <summaries_detail> ]
        editors: [ <summaries_detail> ]
        languages: [ <summaries_detail> ]
        machines: [ <summaries_detail> ]
        operating_system: [ <summaries_detail> ]
        projects: [ <summaries_detail> ]
        grand_total: <summaries_detail>
        range: {
            date: <date string>
            start: <datetime string>
            end: <datetime string>
            text: <string>
            timezone: <string>
        }
    }

    summaries := {
        data: [ <summaries_data> ]
        start: <datetime string>
        end: <datetime string>
    }

    // start, end, project
    project_summaries := {
        data: [ <summaries_data> ]
        start: <datetime string>
        end: <datetime string>
    }

    datetime string := "YYYY'-'MM'-'DD'T'HH':'mi':'ss'Z'"    // e.g. 2021-02-22T14:59:59Z
*/

#[derive(Debug, Deserialize, Serialize)]
pub struct SummariesDetail {
    pub digital: String,
    pub hours: i32,
    pub minutes: i32,
    #[serde(default)]
    pub seconds: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub percent: f32,
    pub text: String,
    pub total_seconds: f32,
    pub machine_name_id: Option<String>
}

impl Default for SummariesDetail {
    fn default() -> Self {
        SummariesDetail {
             digital: "".into(),
            hours: 0,
            minutes: 0,
            seconds: 0,
            name: "".into(),
            percent: 0.0,
            text: "".into(),
            total_seconds: 0.0,
            machine_name_id: Some("".into()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RangeData {
    pub date: String,
    pub start: String,
    pub end: String,
    pub text: String,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SummariesData {
    #[serde(default)]
    pub branches: Vec<SummariesDetail>,
    #[serde(default)]
    pub entities: Vec<SummariesDetail>,
    pub categories: Vec<SummariesDetail>,
    pub dependencies: Vec<SummariesDetail>,
    pub editors: Vec<SummariesDetail>,
    pub languages: Vec<SummariesDetail>,
    pub machines: Vec<SummariesDetail>,
    #[serde(default)]
    pub operating_system: Vec<SummariesDetail>,
    #[serde(default)]
    pub projects: Vec<SummariesDetail>,
    pub grand_total: SummariesDetail,
    pub range: RangeData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Summaries {
    pub data: Vec<SummariesData>,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SummariesAll {
    pub summaries: Summaries,
    pub projects: HashMap<String, Summaries>,
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
