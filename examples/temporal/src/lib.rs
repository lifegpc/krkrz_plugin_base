use std::str::FromStr;

use krkrz_plugin_base::param::TypeError;
use krkrz_plugin_base::{tp_stub::*, *};
use serde::Deserialize;
use temporal_rs::TemporalError;

static mut GLOBAL_REF_COUNT_AT_INIT: tjs_int = 0;
generate_origin_static_block!(temporal);

mod locales;

struct Instant(temporal_rs::Instant);

enum Error {
    Temporal(TemporalError),
    Type(TypeError),
    Error(String),
    DerseError(de::DeserError),
}

impl From<TemporalError> for Error {
    fn from(value: TemporalError) -> Self {
        Self::Temporal(value)
    }
}

impl From<TypeError> for Error {
    fn from(value: TypeError) -> Self {
        Self::Type(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Error(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Error(value.to_owned())
    }
}

impl From<de::DeserError> for Error {
    fn from(value: de::DeserError) -> Self {
        Self::DerseError(value)
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> From<tp_stub::GetItemError<E>> for Error {
    fn from(value: tp_stub::GetItemError<E>) -> Self {
        Self::Error(format!("{}", value))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Temporal(t) => t.fmt(f),
            Self::Type(t) => t.fmt(f),
            Self::Error(t) => t.fmt(f),
            Self::DerseError(t) => t.fmt(f),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FractionalSecondDigits {
    Auto,
    Digit(u8),
}

impl FractionalSecondDigits {
    fn to_precision(&self) -> Result<temporal_rs::parsers::Precision, Error> {
        match self {
            Self::Auto => Ok(temporal_rs::parsers::Precision::Auto),
            Self::Digit(d) => {
                if *d > 9 {
                    return Err("fractionalSecondDigits must be between 0 and 9".into());
                }
                Ok(temporal_rs::parsers::Precision::Digit(*d))
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum RoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

impl RoundingMode {
    fn to_rounding_mode(&self) -> temporal_rs::options::RoundingMode {
        match self {
            Self::Ceil => temporal_rs::options::RoundingMode::Ceil,
            Self::Floor => temporal_rs::options::RoundingMode::Floor,
            Self::Expand => temporal_rs::options::RoundingMode::Expand,
            Self::Trunc => temporal_rs::options::RoundingMode::Trunc,
            Self::HalfCeil => temporal_rs::options::RoundingMode::HalfCeil,
            Self::HalfFloor => temporal_rs::options::RoundingMode::HalfFloor,
            Self::HalfExpand => temporal_rs::options::RoundingMode::HalfExpand,
            Self::HalfTrunc => temporal_rs::options::RoundingMode::HalfTrunc,
            Self::HalfEven => temporal_rs::options::RoundingMode::HalfEven,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum Unit {
    Auto,
    Nanosecond,
    Microsecond,
    Millisecond,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl Unit {
    fn to_unit(&self) -> temporal_rs::options::Unit {
        match self {
            Self::Auto => temporal_rs::options::Unit::Auto,
            Self::Nanosecond => temporal_rs::options::Unit::Nanosecond,
            Self::Microsecond => temporal_rs::options::Unit::Microsecond,
            Self::Millisecond => temporal_rs::options::Unit::Millisecond,
            Self::Second => temporal_rs::options::Unit::Second,
            Self::Minute => temporal_rs::options::Unit::Minute,
            Self::Hour => temporal_rs::options::Unit::Hour,
            Self::Day => temporal_rs::options::Unit::Day,
            Self::Week => temporal_rs::options::Unit::Week,
            Self::Month => temporal_rs::options::Unit::Month,
            Self::Year => temporal_rs::options::Unit::Year,
        }
    }
}

#[Tjs2Class]
#[tjs(new = "Temporal.Instant")]
impl Instant {
    fn new(nanoseconds: i128) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::try_new(nanoseconds)?))
    }

    #[allow(non_upper_case_globals)]
    fn from(info: &mut tTJSVariant) -> Result<Self, Error> {
        match info.typ() {
            tTJSVariantType_tvtString => {
                let s = String::to_param(info)?;
                Ok(Self(temporal_rs::Instant::from_str(&s)?))
            }
            tTJSVariantType_tvtObject => {
                let obj = info.as_object_no_add_ref();
                if obj.is_null() {
                    throw_null_access();
                }
                if unsafe {
                    (*obj).is_instance_of(
                        0,
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        tjs_w!("Date"),
                        obj,
                    )
                } == TJS_S_TRUE
                {
                    // is Date
                    let mut var = tTJSVariant::new();
                    let hr = unsafe {
                        (*obj).func_call(
                            0,
                            tjs_w!("getTime"),
                            std::ptr::null_mut(),
                            &mut var,
                            0,
                            std::ptr::null_mut(),
                            obj,
                        )
                    };
                    if TJS_FAILED(hr) {
                        return Err("failed to get time from Date.".into());
                    }
                    let time = i64::to_param(&mut var)?;
                    return Ok(Self(temporal_rs::Instant::from_epoch_milliseconds(time)?));
                }
                let hr = unsafe {
                    (*obj).is_instance_of(
                        0,
                        std::ptr::null(),
                        std::ptr::null_mut(),
                        tjs_w!("Instant"),
                        obj,
                    )
                };
                if hr != TJS_S_TRUE {
                    return Err(TypeError("Instant or Date").into());
                }
                let mut nano = tTJSVariant::new();
                let hr = unsafe {
                    (*obj).prop_get(
                        0,
                        tjs_w!("epochNanoseconds"),
                        std::ptr::null_mut(),
                        &mut nano,
                        obj,
                    )
                };
                if TJS_FAILED(hr) {
                    return Err("failed to get nanoseconds from Instant.".into());
                }
                let nanoseconds = i128::to_param(&mut nano)?;
                Ok(Self(temporal_rs::Instant::try_new(nanoseconds)?))
            }
            _ => Err(TypeError("string or Instant or Date").into()),
        }
    }

    #[tjs(case = camel)]
    fn from_epoch_milliseconds(epoch_milliseconds: i64) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::from_epoch_milliseconds(
            epoch_milliseconds,
        )?))
    }

    #[tjs(case = camel)]
    fn from_epoch_nanoseconds(epoch_nanoseconds: i128) -> Result<Self, TemporalError> {
        Ok(Self(temporal_rs::Instant::try_new(epoch_nanoseconds)?))
    }

    #[tjs(case = camel)]
    fn get_epoch_milliseconds(&self) -> i64 {
        self.0.epoch_milliseconds()
    }

    #[tjs(case = camel)]
    fn get_epoch_nanoseconds(&self) -> i128 {
        self.0.epoch_nanoseconds().0
    }

    fn add(&self, duration: &mut tTJSVariant) -> Result<tTJSVariant, Error> {
        let v = if Duration::is_instance_of2(duration) {
            let dur = Duration::get_ref2(duration);
            self.0.add(&dur.0)?
        } else {
            let dur = Duration::from(duration)?;
            self.0.add(&dur.0)?
        };
        let new_instant = Self(v);
        let n = Instant::create(new_instant);
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    #[tjs(case = camel)]
    fn to_string(&self, options: Option<&mut tTJSVariant>) -> Result<String, Error> {
        let mut fractional_second_digits = FractionalSecondDigits::Auto;
        let mut rounding_mode = RoundingMode::Trunc;
        let mut smallest_unit: Option<Unit> = None;
        let mut time_zone = None;
        let config_obj = match options.as_ref() {
            Some(c) => {
                let a = c.as_object_no_add_ref();
                if a.is_null() {
                    return Err("options is not a object.".into());
                }
                Some(unsafe { &mut (*a) })
            }
            None => None,
        };
        if let Some(c) = config_obj {
            c.with_get_serde("fractionalSecondDigits", |s| {
                fractional_second_digits = s;
            })?;
            c.with_get_serde("roundingMode", |s| {
                rounding_mode = s;
            })?;
            c.with_get_serde("smallestUnit", |s| {
                smallest_unit = Some(s);
            })?;
            let mut tz = c.get("timeZone")?;
            if let Some(tz) = tz.as_mut() {
                if tz.is_string() {
                    time_zone = Some(temporal_rs::TimeZone::try_from_str(&String::to_param(tz)?)?);
                } else if ZonedDateTime::is_instance_of2(tz) {
                    let zdt = ZonedDateTime::get_ref2(tz);
                    time_zone = Some(zdt.0.time_zone().clone());
                } else {
                    return Err(TypeError("string or ZonedDateTime").into());
                }
            }
        }
        Ok(self.0.to_ixdtf_string(
            time_zone,
            temporal_rs::options::ToStringRoundingOptions {
                precision: fractional_second_digits.to_precision()?,
                rounding_mode: Some(rounding_mode.to_rounding_mode()),
                smallest_unit: smallest_unit.map(|u| u.to_unit()),
            },
        )?)
    }

    #[tjs(case = camel, serde)]
    fn to_locale_string(
        &self,
        locales: Option<locales::Locales>,
        options: Option<locales::ToLocaleStringOptions>,
    ) -> Result<String, Error> {
        locales::format_instant(&self.0, locales, options)
    }
}

impl Drop for Instant {
    fn drop(&mut self) {
        log!("Instant: dropped {:?}", self.0)
    }
}

struct Duration(temporal_rs::Duration);

#[derive(Deserialize)]
struct DurationFromInfo {
    years: Option<i64>,
    months: Option<i64>,
    weeks: Option<i64>,
    days: Option<i64>,
    hours: Option<i64>,
    minutes: Option<i64>,
    seconds: Option<i64>,
    milliseconds: Option<i64>,
    microseconds: Option<i128>,
    nanoseconds: Option<i128>,
}

#[Tjs2Class]
#[tjs(new = "Temporal.Duration")]
impl Duration {
    fn new(
        years: Option<i64>,
        months: Option<i64>,
        weeks: Option<i64>,
        days: Option<i64>,
        hours: Option<i64>,
        minutes: Option<i64>,
        seconds: Option<i64>,
        milliseconds: Option<i64>,
        microseconds: Option<i128>,
        nanoseconds: Option<i128>,
    ) -> Result<Self, Error> {
        Ok(Self(temporal_rs::Duration::new(
            years.unwrap_or(0),
            months.unwrap_or(0),
            weeks.unwrap_or(0),
            days.unwrap_or(0),
            hours.unwrap_or(0),
            minutes.unwrap_or(0),
            seconds.unwrap_or(0),
            milliseconds.unwrap_or(0),
            microseconds.unwrap_or(0),
            nanoseconds.unwrap_or(0),
        )?))
    }

    #[allow(non_upper_case_globals)]
    fn from(info: &mut tTJSVariant) -> Result<Self, Error> {
        match info.typ() {
            tTJSVariantType_tvtObject => {
                if Duration::is_instance_of2(info) {
                    let inf = Duration::get_ref2(info);
                    return Ok(Self(inf.0.clone()));
                }
                let info = de::from::<DurationFromInfo>(info)?;
                Ok(Self(temporal_rs::Duration::new(
                    info.years.unwrap_or(0),
                    info.months.unwrap_or(0),
                    info.weeks.unwrap_or(0),
                    info.days.unwrap_or(0),
                    info.hours.unwrap_or(0),
                    info.minutes.unwrap_or(0),
                    info.seconds.unwrap_or(0),
                    info.milliseconds.unwrap_or(0),
                    info.microseconds.unwrap_or(0),
                    info.nanoseconds.unwrap_or(0),
                )?))
            }
            tTJSVariantType_tvtString => {
                let s = String::to_param(info)?;
                Ok(Self(temporal_rs::Duration::from_str(&s)?))
            }
            _ => Err(TypeError("string or Duration or Dict").into()),
        }
    }
}

struct Now {}

#[Tjs2Class]
impl Now {
    #[tjs(static_method)]
    fn instant() -> Result<tTJSVariant, Error> {
        let now = temporal_rs::now::Now::new(temporal_rs::sys::LocalHostSystem).instant()?;
        let n = Instant::create(Instant(now));
        let re = tTJSVariant::from(n);
        unsafe { (*n).release() };
        Ok(re)
    }

    #[tjs(static_method, case = camel)]
    fn time_zone_id() -> Result<String, Error> {
        let now = temporal_rs::now::Now::new(temporal_rs::sys::LocalHostSystem);
        let tz = now.time_zone()?;
        Ok(tz.identifier()?)
    }
}

struct ZonedDateTime(temporal_rs::ZonedDateTime);

#[Tjs2Class]
#[tjs(new = "Temporal.ZonedDateTime")]
impl ZonedDateTime {
    fn new(
        epoch_nanoseconds: i128,
        time_zone: String,
        calendar: Option<String>,
    ) -> Result<Self, Error> {
        let calendar = match calendar {
            Some(s) => temporal_rs::Calendar::try_from_utf8(s.as_bytes())?,
            None => temporal_rs::Calendar::ISO,
        };
        let time_zone = temporal_rs::TimeZone::try_from_str(&time_zone)?;
        Ok(Self(temporal_rs::ZonedDateTime::try_new(
            epoch_nanoseconds,
            time_zone,
            calendar,
        )?))
    }
}

struct Temporal {}

#[Tjs2Class]
impl Temporal {
    #[tjs(static_member, case = pascal)]
    fn instant() -> tTJSVariant {
        let class = Instant::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }

    #[tjs(static_member, case = pascal)]
    fn duration() -> tTJSVariant {
        let class = Duration::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }

    #[tjs(static_member, case = pascal)]
    fn now() -> tTJSVariant {
        let class = Now::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }

    #[tjs(static_member, case = pascal)]
    fn zoned_date_time() -> tTJSVariant {
        let class = ZonedDateTime::create_native_class().1;
        let re = tTJSVariant::from(class);
        unsafe { (*class).release() };
        re
    }
}

#[unsafe(export_name = "V2Link")]
unsafe extern "system" fn v2_link(exporter: *mut iTVPFunctionExporter) -> i32 {
    unsafe { TVPInitImportStub(exporter) };
    let temporal = Temporal::create_native_class().1;
    register_var!(temporal);
    unsafe { GLOBAL_REF_COUNT_AT_INIT = TVPPluginGlobalRefCount };
    0
}

#[unsafe(export_name = "V2Unlink")]
unsafe extern "system" fn v2_unlink() -> i32 {
    if unsafe { TVPPluginGlobalRefCount > GLOBAL_REF_COUNT_AT_INIT } {
        log!("[temporal]Can not unlink plugin");
        return TJS_E_FAIL;
    }
    unregister_var!(temporal);
    log!("[temporal]unlinked plugin");
    unsafe { TVPUninitImportStub() };
    0
}
