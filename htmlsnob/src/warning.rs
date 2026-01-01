use serde::Deserialize;

use crate::ast::Area;

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum WarningSeverity {
    #[default]
    ERROR = 1,
    WARNING = 2,
    INFORMATION = 3,
    HINT = 4,
}

#[derive(Debug)]
pub struct Warning {
    pub name: String,
    pub severity: WarningSeverity,
    pub message: String,
    pub areas: Vec<Area>,
}

impl Warning {
    pub fn new(
        name: &str,
        kind: &str,
        areas: &[Area],
        message: &str,
        severity: WarningSeverity,
    ) -> Warning {
        let name_or_kind = if name.is_empty() { kind } else { name };

        Warning {
            name: name_or_kind.to_string(),
            severity,
            message: message.to_string(),
            areas: areas.to_vec(),
        }
    }

    pub fn from_areas(
        name: &str,
        kind: &str,
        areas: &[Area],
        message: &str,
        severity: WarningSeverity,
    ) -> Warning {
        Warning::new(name, kind, areas, message, severity)
    }

    pub fn from_area(
        name: &str,
        kind: &str,
        area: Area,
        message: &str,
        severity: WarningSeverity,
    ) -> Warning {
        Warning::from_areas(name, kind, &[area], message, severity)
    }
}
