#[cfg(feature = "locales")]
mod icu_impl {
    pub use icu::calendar::Date;
    pub use icu::datetime::DateTimeFormatter;
    pub use icu::datetime::fieldsets::builder::{DateFields, FieldSetBuilder, ZoneStyle};
    pub use icu::datetime::fieldsets::enums::CompositeFieldSet;
    pub use icu::datetime::options::{Length, SubsecondDigits, TimePrecision, YearStyle};
    pub use icu::locale::Locale;
    pub use icu::locale::extensions::unicode::{Value, key};
    pub use icu::time::zone::{IanaParser, UtcOffset};
    pub use icu::time::{Time, ZonedDateTime as IcuZonedDateTime};
}
#[cfg(feature = "locales")]
use icu_impl::*;
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize)]
#[serde(untagged)]
#[cfg_attr(not(feature = "locales"), allow(dead_code))]
pub enum Locales {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum LocaleMatcher {
    Lookup,
    #[serde(rename = "best fit")]
    BestFit,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum FormatMatcher {
    Basic,
    #[serde(rename = "best fit")]
    BestFit,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum HourCycle {
    H11,
    H12,
    H23,
    H24,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DateTimeStyle {
    Full,
    Long,
    Medium,
    Short,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum TextWidth {
    Long,
    Short,
    Narrow,
}

#[derive(Deserialize, Clone, Copy)]
pub enum Numeric2Digit {
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "2-digit")]
    TwoDigit,
}

#[derive(Deserialize, Clone, Copy)]
pub enum MonthFormat {
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "2-digit")]
    TwoDigit,
    #[serde(rename = "long")]
    Long,
    #[serde(rename = "short")]
    Short,
    #[serde(rename = "narrow")]
    Narrow,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum TimeZoneNameStyle {
    Long,
    Short,
    ShortOffset,
    LongOffset,
    ShortGeneric,
    LongGeneric,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(feature = "locales"), allow(dead_code))]
pub struct ToLocaleStringOptions {
    #[allow(dead_code)]
    pub locale_matcher: Option<LocaleMatcher>,
    #[allow(dead_code)]
    pub format_matcher: Option<FormatMatcher>,
    pub time_zone: Option<String>,
    pub calendar: Option<String>,
    pub numbering_system: Option<String>,
    pub hour12: Option<bool>,
    pub hour_cycle: Option<HourCycle>,
    pub date_style: Option<DateTimeStyle>,
    pub time_style: Option<DateTimeStyle>,
    pub weekday: Option<TextWidth>,
    pub era: Option<TextWidth>,
    pub year: Option<Numeric2Digit>,
    pub month: Option<MonthFormat>,
    pub day: Option<Numeric2Digit>,
    pub day_period: Option<TextWidth>,
    pub hour: Option<Numeric2Digit>,
    pub minute: Option<Numeric2Digit>,
    pub second: Option<Numeric2Digit>,
    pub fractional_second_digits: Option<u8>,
    pub time_zone_name: Option<TimeZoneNameStyle>,
}

#[cfg_attr(not(feature = "locales"), allow(dead_code))]
impl ToLocaleStringOptions {
    fn has_style(&self) -> bool {
        self.date_style.is_some() || self.time_style.is_some()
    }

    fn has_components(&self) -> bool {
        self.weekday.is_some()
            || self.era.is_some()
            || self.year.is_some()
            || self.month.is_some()
            || self.day.is_some()
            || self.day_period.is_some()
            || self.hour.is_some()
            || self.minute.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some()
            || self.time_zone_name.is_some()
    }

    fn validate(&self) -> Result<(), Error> {
        if let Some(d) = self.fractional_second_digits {
            if !(1..=3).contains(&d) {
                return Err("fractionalSecondDigits must be between 1 and 3".into());
            }
        }
        if self.has_style() && self.has_components() {
            return Err(
                "dateStyle and timeStyle cannot be used together with other date/time component options"
                    .into(),
            );
        }
        Ok(())
    }
}

#[cfg(feature = "locales")]
fn default_system_locale() -> Locale {
    sys_locale::get_locale()
        .and_then(|s| Locale::try_from_str(&s).ok())
        .unwrap_or_else(|| Locale::try_from_str("en").unwrap())
}

#[cfg(feature = "locales")]
fn resolve_locale(
    locales: Option<Locales>,
    options: &ToLocaleStringOptions,
) -> Result<Locale, Error> {
    let mut locale = match locales {
        Some(Locales::Single(tag)) => Locale::try_from_str(&tag)
            .map_err(|e| Error::Error(format!("Invalid locale tag '{}': {}", tag, e)))?,
        Some(Locales::Multiple(tags)) => {
            let mut chosen = None;
            for tag in &tags {
                let parsed = Locale::try_from_str(tag)
                    .map_err(|e| Error::Error(format!("Invalid locale tag '{}': {}", tag, e)))?;
                if chosen.is_none() {
                    chosen = Some(parsed);
                }
            }
            chosen.unwrap_or_else(default_system_locale)
        }
        None => default_system_locale(),
    };

    if let Some(cal) = &options.calendar {
        let val = Value::try_from_str(cal)
            .map_err(|e| Error::Error(format!("Invalid calendar '{}': {}", cal, e)))?;
        locale.extensions.unicode.keywords.set(key!("ca"), val);
    }

    if let Some(nu) = &options.numbering_system {
        let val = Value::try_from_str(nu)
            .map_err(|e| Error::Error(format!("Invalid numberingSystem '{}': {}", nu, e)))?;
        locale.extensions.unicode.keywords.set(key!("nu"), val);
    }

    if let Some(hc) = options.hour_cycle {
        let hc_str = match hc {
            HourCycle::H11 => "h11",
            HourCycle::H12 => "h12",
            HourCycle::H23 => "h23",
            HourCycle::H24 => "h24",
        };
        locale
            .extensions
            .unicode
            .keywords
            .set(key!("hc"), Value::try_from_str(hc_str).unwrap());
    } else if let Some(hour12) = options.hour12 {
        let hc_str = if hour12 { "h12" } else { "h23" };
        locale
            .extensions
            .unicode
            .keywords
            .set(key!("hc"), Value::try_from_str(hc_str).unwrap());
    }

    Ok(locale)
}

#[cfg(feature = "locales")]
fn build_field_set(options: &ToLocaleStringOptions) -> Result<CompositeFieldSet, Error> {
    let mut builder = FieldSetBuilder::default();

    if options.has_style() {
        if let Some(ds) = options.date_style {
            builder.date_fields = Some(match ds {
                DateTimeStyle::Full => DateFields::YMDE,
                DateTimeStyle::Long | DateTimeStyle::Medium | DateTimeStyle::Short => {
                    DateFields::YMD
                }
            });
            builder.length = Some(match ds {
                DateTimeStyle::Full | DateTimeStyle::Long => Length::Long,
                DateTimeStyle::Medium => Length::Medium,
                DateTimeStyle::Short => Length::Short,
            });
        }
        if let Some(ts) = options.time_style {
            builder.time_precision = Some(match ts {
                DateTimeStyle::Full | DateTimeStyle::Long | DateTimeStyle::Medium => {
                    TimePrecision::Second
                }
                DateTimeStyle::Short => TimePrecision::Minute,
            });
            if builder.length.is_none() {
                builder.length = Some(match ts {
                    DateTimeStyle::Full | DateTimeStyle::Long => Length::Long,
                    DateTimeStyle::Medium => Length::Medium,
                    DateTimeStyle::Short => Length::Short,
                });
            }
            match ts {
                DateTimeStyle::Full => builder.zone_style = Some(ZoneStyle::SpecificLong),
                DateTimeStyle::Long => builder.zone_style = Some(ZoneStyle::SpecificShort),
                DateTimeStyle::Medium | DateTimeStyle::Short => {}
            }
        }
        return builder
            .build_composite()
            .map_err(|e| Error::Error(format!("Invalid format options: {:?}", e)));
    }

    if !options.has_components() {
        // ECMA-402 default for Instant: numeric date + time
        builder.date_fields = Some(DateFields::YMD);
        builder.length = Some(Length::Short);
        builder.time_precision = Some(TimePrecision::Second);
        return builder
            .build_composite()
            .map_err(|e| Error::Error(format!("Invalid format options: {:?}", e)));
    }

    let has_y = options.year.is_some();
    let has_m = options.month.is_some();
    let has_d = options.day.is_some();
    let has_e = options.weekday.is_some();

    builder.date_fields = match (has_y, has_m, has_d, has_e) {
        (false, false, false, false) => None,
        (true, true, true, true) => Some(DateFields::YMDE),
        (true, true, true, false) => Some(DateFields::YMD),
        (false, true, true, true) => Some(DateFields::MDE),
        (false, true, true, false) => Some(DateFields::MD),
        (false, false, true, true) => Some(DateFields::DE),
        (false, false, true, false) => Some(DateFields::D),
        (false, false, false, true) => Some(DateFields::E),
        (true, true, false, false) => Some(DateFields::YM),
        (true, false, false, false) => Some(DateFields::Y),
        (false, true, false, false) => Some(DateFields::M),
        _ => Some(DateFields::YMD),
    };

    if has_y || options.era.is_some() {
        builder.year_style = Some(match (options.year, options.era) {
            (_, Some(_)) => YearStyle::WithEra,
            (Some(Numeric2Digit::TwoDigit), None) => YearStyle::Auto,
            (Some(Numeric2Digit::Numeric), None) | (None, None) => YearStyle::Full,
        });
    }

    if builder.date_fields.is_some() {
        builder.length = Some(
            if matches!(options.month, Some(MonthFormat::Long))
                || matches!(options.weekday, Some(TextWidth::Long))
            {
                Length::Long
            } else if matches!(options.month, Some(MonthFormat::Short))
                || matches!(options.weekday, Some(TextWidth::Short))
            {
                Length::Medium
            } else {
                Length::Short
            },
        );
    }

    let has_time = options.hour.is_some()
        || options.minute.is_some()
        || options.second.is_some()
        || options.fractional_second_digits.is_some()
        || options.day_period.is_some();

    if has_time {
        builder.time_precision = Some(if let Some(d) = options.fractional_second_digits {
            let subsec = match d {
                1 => SubsecondDigits::S1,
                2 => SubsecondDigits::S2,
                3 => SubsecondDigits::S3,
                _ => SubsecondDigits::S3,
            };
            TimePrecision::Subsecond(subsec)
        } else if options.second.is_some() {
            TimePrecision::Second
        } else if options.minute.is_some() {
            TimePrecision::Minute
        } else {
            TimePrecision::Hour
        });
    }

    builder.zone_style = options.time_zone_name.map(|z| match z {
        TimeZoneNameStyle::Long => ZoneStyle::SpecificLong,
        TimeZoneNameStyle::Short => ZoneStyle::SpecificShort,
        TimeZoneNameStyle::ShortOffset => ZoneStyle::LocalizedOffsetShort,
        TimeZoneNameStyle::LongOffset => ZoneStyle::LocalizedOffsetLong,
        TimeZoneNameStyle::ShortGeneric => ZoneStyle::GenericShort,
        TimeZoneNameStyle::LongGeneric => ZoneStyle::GenericLong,
    });

    builder
        .build_composite()
        .map_err(|e| Error::Error(format!("Invalid format options: {:?}", e)))
}

#[cfg(feature = "locales")]
pub fn format_instant(
    instant: &temporal_rs::Instant,
    locales: Option<Locales>,
    options: Option<ToLocaleStringOptions>,
) -> Result<String, Error> {
    let options = options.unwrap_or_default();
    options.validate()?;

    let locale = resolve_locale(locales, &options)?;
    let field_set = build_field_set(&options)?;

    let time_zone = match &options.time_zone {
        Some(tz) => temporal_rs::TimeZone::try_from_str(tz)?,
        None => temporal_rs::now::Now::new(temporal_rs::sys::LocalHostSystem).time_zone()?,
    };

    let zdt = instant.to_zoned_date_time_iso(time_zone)?;

    let date = Date::try_new_iso(zdt.year(), zdt.month(), zdt.day())
        .map_err(|e| Error::Error(format!("Failed to convert date: {}", e)))?;

    let subsec_ns = (zdt.millisecond() as u32) * 1_000_000
        + (zdt.microsecond() as u32) * 1_000
        + (zdt.nanosecond() as u32);
    let time = Time::try_new(zdt.hour(), zdt.minute(), zdt.second(), subsec_ns)
        .map_err(|e| Error::Error(format!("Failed to convert time: {}", e)))?;

    let tz_id = zdt.time_zone().identifier()?;
    let offset_secs = (zdt.offset_nanoseconds() / 1_000_000_000) as i32;
    let offset = UtcOffset::try_from_seconds(offset_secs)
        .map_err(|e| Error::Error(format!("Invalid UTC offset: {:?}", e)))?;

    let icu_tz = IanaParser::new().parse(&tz_id);
    let zone_info = icu_tz
        .with_offset(Some(offset))
        .at_date_time(icu::time::DateTime { date, time });

    let icu_zdt = IcuZonedDateTime {
        date,
        time,
        zone: zone_info,
    };

    let dtf = DateTimeFormatter::try_new(locale.into(), field_set)
        .map_err(|e| Error::Error(format!("Failed to create DateTimeFormatter: {}", e)))?;

    Ok(dtf.format(&icu_zdt).to_string())
}

#[cfg(not(feature = "locales"))]
pub fn format_instant(
    _instant: &temporal_rs::Instant,
    _locales: Option<Locales>,
    _options: Option<ToLocaleStringOptions>,
) -> Result<String, Error> {
    Err("locales feature is not enabled".into())
}
