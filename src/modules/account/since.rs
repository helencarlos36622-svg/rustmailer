// Copyright © 2025-2026 rustmailer.com
// Licensed under RustMailer License Agreement v1.0
// Unauthorized copying, modification, or distribution is prohibited.

use crate::{
    modules::error::{code::ErrorCode, RustMailerResult},
    raise_error,
};
use chrono::{Datelike, Days, Local, Months, NaiveDate, TimeZone, Utc};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct DateSince {
    /// Absolute date boundary in ISO 8601 format (YYYY-MM-DD)
    ///
    /// ### Validation Rules
    /// - Must match exact format `^\d{4}-\d{2}-\d{2}$`
    /// - Date must be logically valid (e.g. no 2025-05-01)
    ///
    /// ### Example
    /// ```json
    /// {
    ///   "fixed": "2025-05-01"
    /// }
    /// ```
    #[oai(validator(pattern = r"^\d{4}-\d{2}-\d{2}$"))]
    pub fixed: Option<String>,
    /// Relative time period from current date
    ///
    /// ### Constraints
    /// - Value must be ≥ 1
    /// - Units support day/month/year granularity
    ///
    /// ### Example
    /// ```json
    /// {
    ///   "relative": {
    ///     "unit": "Days",
    ///     "value": 7
    ///   }
    /// }
    /// ```
    pub relative: Option<RelativeDate>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Enum)]
pub enum Unit {
    #[default]
    Days,
    Months,
    Years,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Object)]
pub struct RelativeDate {
    /// The time unit to use for the offset (days, months, or years)
    pub unit: Unit,
    /// The quantity of time units to offset (must be a positive integer)
    #[oai(validator(minimum(value = "1")))]
    pub value: u32,
}

impl RelativeDate {
    pub fn validate_date(&self) -> RustMailerResult<()> {
        if self.value == 0 {
            return Err(raise_error!(
                "Value must be greater than 0".into(),
                ErrorCode::InvalidParameter
            ));
        }

        let now = Local::now();
        let date = match self.unit {
            Unit::Days => now.checked_sub_days(Days::new(self.value as u64)),
            Unit::Months => now.checked_sub_months(Months::new(self.value)),
            Unit::Years => now.checked_sub_months(Months::new(self.value * 12)),
        };

        let date = date.ok_or_else(|| {
            raise_error!(
                "Invalid date: the calculated date is earlier than 1970 or an overflow occurred."
                    .into(),
                ErrorCode::InvalidParameter
            )
        })?;

        let naive_date = date.date_naive();

        // Check if the date is before 1970
        if naive_date.year() < 1970 {
            return Err(raise_error!(
                format!(
                    "Date cannot be earlier than 1970-01-01. Provided: '{}'",
                    naive_date
                ),
                ErrorCode::InvalidParameter
            ));
        }

        Ok(())
    }

    fn compute_date(&self) -> RustMailerResult<chrono::DateTime<Local>> {
        if self.value == 0 {
            return Err(raise_error!(
                "Value must be greater than 0".into(),
                ErrorCode::InvalidParameter
            ));
        }

        let now = Local::now();
        let date = match self.unit {
            Unit::Days => now.checked_sub_days(Days::new(self.value as u64)),
            Unit::Months => now.checked_sub_months(Months::new(self.value)),
            Unit::Years => now.checked_sub_months(Months::new(self.value * 12)),
        };

        let date = date.ok_or_else(|| {
            raise_error!(
                "Invalid date: the calculated date is earlier than 1970 or an overflow occurred."
                    .into(),
                ErrorCode::InvalidParameter
            )
        })?;

        let naive_date = date.date_naive();
        if naive_date.year() < 1970 {
            return Err(raise_error!(
                format!(
                    "Date cannot be earlier than 1970-01-01. Provided: '{}'",
                    naive_date
                ),
                ErrorCode::InvalidParameter
            ));
        }

        Ok(date)
    }

    pub fn calculate_date(&self) -> RustMailerResult<String> {
        let date = self.compute_date()?;
        Ok(date.format("%d-%b-%Y").to_string())
    }

    pub fn calculate_gmail_date(&self) -> RustMailerResult<String> {
        let date = self.compute_date()?;
        Ok(date.format("%Y/%m/%d").to_string())
    }

    pub fn calculate_outlook_date(&self) -> RustMailerResult<String> {
        let date = self.compute_date()?;
        let dt_utc = date.with_timezone(&Utc);
        Ok(dt_utc.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
    }
}

impl DateSince {
    pub fn validate(&self) -> RustMailerResult<()> {
        match (&self.fixed, &self.relative) {
            // If only `relative` is provided
            (None, Some(r)) => {
                r.validate_date()?;
            }
            // If only `fixed` is provided
            (Some(fixed), None) => {
                self.validate_fixed_date(fixed)?;
            }
            // If both or neither are provided
            _ => {
                return Err(raise_error!(
                    "Invalid input: You must provide either 'fixed' or 'relative', but not both."
                        .to_string(),
                    ErrorCode::InvalidParameter
                ));
            }
        }
        Ok(())
    }

    fn validate_fixed_date(&self, fixed: &str) -> RustMailerResult<()> {
        // Try to parse the input string as YYYY-MM-DD
        let date = NaiveDate::parse_from_str(fixed, "%Y-%m-%d").map_err(|_| {
            raise_error!(
                format!(
                "Invalid date format. Expected 'YYYY-MM-DD'. Example: '2024-11-19'. Provided: '{}'",
                fixed
            ),
                ErrorCode::InvalidParameter
            )
        })?;

        let now = Utc::now().date_naive();

        // Check if the date is in the future
        if date >= now {
            return Err(raise_error!(
                format!(
                    "Date cannot be in the future. Provided: '{}', Today: '{}'",
                    fixed,
                    now.format("%Y-%m-%d")
                ),
                ErrorCode::InvalidParameter
            ));
        }

        // Check if the date is before 1970
        if date.year() < 1970 {
            return Err(raise_error!(
                format!(
                    "Date cannot be earlier than 1970-01-01. Provided: '{}'",
                    fixed
                ),
                ErrorCode::InvalidParameter
            ));
        }

        Ok(())
    }

    fn format_user_date(&self, fixed: &str) -> RustMailerResult<String> {
        let date = NaiveDate::parse_from_str(fixed, "%Y-%m-%d").map_err(|_| {
            raise_error!(
                format!(
                "Invalid date format. Expected 'YYYY-MM-DD'. Example: '2024-11-19'. Provided: '{}'",
                fixed
            ),
                ErrorCode::InvalidParameter
            )
        })?;
        // Format the date into "%d-%b-%Y" format
        Ok(date.format("%d-%b-%Y").to_string())
    }

    pub fn format_for_gmail(&self, fixed: &str) -> RustMailerResult<String> {
        let date = NaiveDate::parse_from_str(fixed, "%Y-%m-%d").map_err(|_| {
            raise_error!(
                format!(
                "Invalid date format. Expected 'YYYY-MM-DD'. Example: '2024-11-19'. Provided: '{}'",
                fixed
            ),
                ErrorCode::InvalidParameter
            )
        })?;

        Ok(date.format("%Y/%m/%d").to_string())
    }

    pub fn format_for_outlook(&self, fixed: &str) -> RustMailerResult<String> {
        let date = NaiveDate::parse_from_str(fixed, "%Y-%m-%d").map_err(|_| {
            raise_error!(
                format!(
                "Invalid date format. Expected 'YYYY-MM-DD'. Example: '2024-11-19'. Provided: '{}'",
                fixed
            ),
                ErrorCode::InvalidParameter
            )
        })?;
        let naive_dt = date.and_hms_opt(0, 0, 0).ok_or_else(|| {
            raise_error!(
                format!("Invalid time components for date '{}'", fixed),
                ErrorCode::InvalidParameter
            )
        })?;
        let dt_utc = Utc.from_utc_datetime(&naive_dt);
        Ok(dt_utc.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
    }

    pub fn since_date(&self) -> RustMailerResult<String> {
        // Handle the case where only one of `fixed` or `relative` is provided
        if let Some(r) = &self.relative {
            // If `relative` is provided, calculate the date
            r.calculate_date()
        } else if let Some(f) = &self.fixed {
            // If `fixed` is provided, format the date
            self.format_user_date(f)
        } else {
            // If neither is provided, return an error
            Err(raise_error!(
                "You must provide either a 'fixed' or 'relative' date.".to_string(),
                ErrorCode::InvalidParameter
            ))
        }
    }

    pub fn since_gmail_date(&self) -> RustMailerResult<String> {
        if let Some(r) = &self.relative {
            r.calculate_gmail_date()
        } else if let Some(f) = &self.fixed {
            self.format_for_gmail(f)
        } else {
            Err(raise_error!(
                "You must provide either a 'fixed' or 'relative' date.".to_string(),
                ErrorCode::InvalidParameter
            ))
        }
    }

    pub fn since_outlook_date(&self) -> RustMailerResult<String> {
        if let Some(r) = &self.relative {
            r.calculate_outlook_date()
        } else if let Some(f) = &self.fixed {
            self.format_for_outlook(f)
        } else {
            Err(raise_error!(
                "You must provide either a 'fixed' or 'relative' date.".to_string(),
                ErrorCode::InvalidParameter
            ))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::modules::account::since::{DateSince, RelativeDate, Unit};

    #[test]
    fn test1() {
        let e = DateSince {
            fixed: Some("2014-09-12".to_string()),
            relative: None,
        };

        e.validate().unwrap();

        println!("{}", e.since_date().unwrap());

        let e = DateSince {
            fixed: None,
            relative: Some(RelativeDate {
                unit: Unit::Days,
                value: 1,
            }),
        };

        e.validate().unwrap();

        println!("{}", e.since_date().unwrap());
    }


    #[test]
    fn test2() {
        let e = DateSince {
            fixed: Some("2014-09-12".to_string()),
            relative: None,
        };

        e.validate().unwrap();

        println!("{}", e.since_outlook_date().unwrap());

        let e = DateSince {
            fixed: None,
            relative: Some(RelativeDate {
                unit: Unit::Days,
                value: 1,
            }),
        };

        e.validate().unwrap();

        println!("{}", e.since_outlook_date().unwrap());
    }
}
