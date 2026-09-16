/// The config for the cron scheduler
#[derive(Debug)]
pub struct Config<Schedule, Tz> {
    /// The schedule for the cron job
    pub(super) schedule: Schedule,

    pub(super) timezone: Tz,
}

impl<Schedule: Clone, Timezone: Clone> Clone for Config<Schedule, Timezone> {
    fn clone(&self) -> Self {
        Self {
            schedule: self.schedule.clone(),
            timezone: self.timezone.clone(),
        }
    }
}

impl<Schedule, Timezone> Config<Schedule, Timezone> {
    /// Creates a new `CronContext` with the given schedule.
    pub const fn new(schedule: Schedule, timezone: Timezone) -> Self {
        Self { schedule, timezone }
    }

    /// Returns a reference to the schedule, if it exists.
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }

    /// Returns the set timestamp
    pub fn timezone(&self) -> &Timezone {
        &self.timezone
    }
}

#[cfg(feature = "serde")]
mod encoding {
    use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
    use serde::ser::SerializeStruct;
    use serde::{Serialize, Serializer};
    use std::fmt::{self, Display};
    use std::marker::PhantomData;
    use std::str::FromStr;

    use super::*;

    impl<Schedule, Tz> Serialize for Config<Schedule, Tz>
    where
        Schedule: Display,
        Tz: Display,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_struct("Config", 2)?;

            state.serialize_field("schedule", &self.schedule.to_string())?;
            state.serialize_field("timezone", &self.timezone.to_string())?;

            state.end()
        }
    }

    impl<'de, Schedule, Tz> Deserialize<'de> for Config<Schedule, Tz>
    where
        Schedule: FromStr,
        Schedule::Err: fmt::Display,
        Tz: FromStr,
        Tz::Err: Display,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[derive(serde::Deserialize)]
            #[serde(field_identifier, rename_all = "lowercase")]
            enum Field {
                Schedule,
                Timezone,
            }

            struct ConfigVisitor<Schedule, Tz> {
                marker: PhantomData<fn() -> Config<Schedule, Tz>>,
            }

            impl<'de, Schedule, Tz> Visitor<'de> for ConfigVisitor<Schedule, Tz>
            where
                Schedule: FromStr,
                Schedule::Err: fmt::Display,
                Tz: FromStr,
                Tz::Err: fmt::Display,
            {
                type Value = Config<Schedule, Tz>;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    formatter.write_str("struct Config")
                }

                fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
                where
                    V: MapAccess<'de>,
                {
                    let mut schedule: Option<String> = None;
                    let mut timezone: Option<String> = None;

                    while let Some(key) = map.next_key::<Field>()? {
                        match key {
                            Field::Schedule => {
                                if schedule.is_some() {
                                    return Err(de::Error::duplicate_field("schedule"));
                                }
                                schedule = Some(map.next_value()?);
                            }
                            Field::Timezone => {
                                if timezone.is_some() {
                                    return Err(de::Error::duplicate_field("timezone"));
                                }
                                timezone = Some(map.next_value()?);
                            }
                        }
                    }

                    let schedule = schedule.ok_or_else(|| de::Error::missing_field("schedule"))?;
                    let timezone = timezone.ok_or_else(|| de::Error::missing_field("timezone"))?;

                    let schedule = Schedule::from_str(&schedule).map_err(de::Error::custom)?;
                    let timezone = Tz::from_str(&timezone).map_err(de::Error::custom)?;

                    Ok(Config { schedule, timezone })
                }
            }

            const FIELDS: &[&str] = &["schedule", "timezone"];
            deserializer.deserialize_struct(
                "Config",
                FIELDS,
                ConfigVisitor {
                    marker: PhantomData,
                },
            )
        }
    }
}
