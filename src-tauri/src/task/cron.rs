use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct CronExpr {
    pub minutes: Vec<u32>,
    pub hours: Vec<u32>,
    pub days_of_month: Vec<u32>,
    pub months: Vec<u32>,
    pub days_of_week: Vec<u32>,
}

fn parse_field(field: &str, min: u32, max: u32) -> Result<Vec<u32>, String> {
    let field = field.trim();
    let mut values = Vec::new();

    if field == "*" {
        for v in min..=max {
            values.push(v);
        }
        return Ok(values);
    }

    for part in field.split(',') {
        let part = part.trim();
        if part.contains('/') {
            let step_parts: Vec<&str> = part.split('/').collect();
            if step_parts.len() != 2 {
                return Err(format!("Invalid step expression: {}", part));
            }
            let range_str = step_parts[0].trim();
            let step: u32 = step_parts[1].trim().parse().map_err(|_| format!("Invalid step value: {}", step_parts[1]))?;
            if step == 0 {
                return Err("Step value cannot be zero".to_string());
            }
            let (start, end) = if range_str == "*" { (min, max) } else { parse_range(range_str, min, max)? };
            let mut v = start;
            while v <= end {
                values.push(v);
                v += step;
            }
        } else if part.contains('-') {
            let (start, end) = parse_range(part, min, max)?;
            for v in start..=end {
                values.push(v);
            }
        } else {
            let v: u32 = part.parse().map_err(|_| format!("Invalid cron value: {}", part))?;
            if v < min || v > max {
                return Err(format!("Value {} out of range [{}, {}]", v, min, max));
            }
            values.push(v);
        }
    }

    values.sort();
    values.dedup();
    Ok(values)
}

fn parse_range(part: &str, _min: u32, _max: u32) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = part.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid range: {}", part));
    }
    let start: u32 = parts[0].trim().parse().map_err(|_| format!("Invalid range start: {}", parts[0]))?;
    let end: u32 = parts[1].trim().parse().map_err(|_| format!("Invalid range end: {}", parts[1]))?;
    Ok((start, end))
}

pub fn parse_cron(expression: &str) -> Result<CronExpr, String> {
    let parts: Vec<&str> = expression.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(format!(
            "Cron expression must have 5 fields (minute hour day month weekday), got {}",
            parts.len()
        ));
    }

    Ok(CronExpr {
        minutes: parse_field(parts[0], 0, 59)?,
        hours: parse_field(parts[1], 0, 23)?,
        days_of_month: parse_field(parts[2], 1, 31)?,
        months: parse_field(parts[3], 1, 12)?,
        days_of_week: parse_field(parts[4], 0, 7)?,
    })
}

pub fn calc_next_run(cron_expr: &str, from_time: &DateTime<Utc>) -> Result<String, String> {
    let expr = parse_cron(cron_expr)?;

    let mut current = from_time.clone() + Duration::minutes(1);
    current = current.with_second(0).unwrap_or(current);
    current = current.with_nanosecond(0).unwrap_or(current);

    for _ in 0..(366 * 24 * 60) {
        if expr.months.contains(&(current.month())) {
            let month_match_day = expr.days_of_month.contains(&(current.day()));
            let weekday_num = current.weekday().num_days_from_sunday();
            let weekday_match = expr.days_of_week.contains(&weekday_num) || expr.days_of_week.contains(&7);

            let day_match = if expr.days_of_month.iter().any(|&d| d != 0)
                || !expr.days_of_week.is_empty()
            {
                if !expr.days_of_month.iter().any(|&d| d != 0) && !expr.days_of_month.contains(&0) {
                    weekday_match
                } else if expr.days_of_week.iter().any(|&d| d != 0) {
                    month_match_day || weekday_match
                } else {
                    month_match_day
                }
            } else {
                month_match_day
            };

            if day_match
                && expr.hours.contains(&(current.hour()))
                && expr.minutes.contains(&(current.minute()))
            {
                return Ok(current.to_rfc3339());
            }
        }
        current = current + Duration::minutes(1);
    }

    Err("Cannot find next matching time within one year".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_every_minute() {
        let expr = parse_cron("* * * * *").unwrap();
        assert_eq!(expr.minutes.len(), 60);
        assert_eq!(expr.hours.len(), 24);
    }

    #[test]
    fn test_parse_specific_time() {
        let expr = parse_cron("30 9 * * *").unwrap();
        assert_eq!(expr.minutes, vec![30]);
        assert_eq!(expr.hours, vec![9]);
    }
}