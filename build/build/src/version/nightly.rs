use crate::prelude::*;

use crate::version;

use chrono::Datelike;
use semver::Prerelease;
use semver::Version;



/// Parsed nightly build [prerelease](https://semver.org/#spec-item-9) piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NightlyPrerelease {
    pub year:  u64,
    pub month: u64,
    pub day:   u64,
    pub index: Option<u64>,
}

impl NightlyPrerelease {
    pub fn new(date: chrono::NaiveDate, index: Option<u64>) -> Self {
        Self { year: date.year() as u64, month: date.month() as u64, day: date.day() as u64, index }
    }

    /// Generate a next nightly build version for the given date.
    ///
    /// ```
    /// use enso_build::version::nightly::NightlyPrerelease;
    /// let first = NightlyPrerelease { year: 2020, month: 1, day: 1, index: None };
    /// let second = first.next();
    /// assert_eq!(second, NightlyPrerelease { year: 2020, month: 1, day: 1, index: Some(1) });
    /// let third = second.next();
    /// assert_eq!(third, NightlyPrerelease { year: 2020, month: 1, day: 1, index: Some(2) });
    /// ```
    pub fn next(mut self) -> Self {
        self.index = Some(self.index.unwrap_or(0) + 1);
        self
    }
}

impl TryFrom<&Prerelease> for NightlyPrerelease {
    type Error = anyhow::Error;

    #[context("Failed to parse nightly version prerelease: `{prerelease}`.")]
    fn try_from(prerelease: &Prerelease) -> std::result::Result<Self, Self::Error> {
        let prerelease = prerelease.as_str();
        let identifiers = prerelease.split('.').collect_vec();
        ensure!(
            identifiers.get(0).contains(&&version::NIGHTLY_BUILD_PREFIX),
            "Not a nightly build."
        );
        ensure!(identifiers.len() == 4 || identifiers.len() == 5, "Wrong number of identifiers.");
        let year = identifiers.get(1).context("Missing year")?.parse2().context("Invalid year")?;
        let month =
            identifiers.get(2).context("Missing month")?.parse2().context("Invalid month")?;
        let day = identifiers.get(3).context("Missing day")?.parse2().context("Invalid day")?;
        let index =
            identifiers.get(4).map(|index| index.parse2()).transpose().context("Invalid index")?;
        Ok(Self { year, month, day, index })
    }
}

impl Display for NightlyPrerelease {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self { year, month, day, index } = self;
        write!(f, "{}.{}.{}.{}", version::NIGHTLY_BUILD_PREFIX, year, month, day)?;
        if let Some(index) = index {
            write!(f, ".{}", index)?;
        }
        Ok(())
    }
}

impl TryInto<Prerelease> for NightlyPrerelease {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Prerelease, Self::Error> {
        let as_string = self.to_string();
        Prerelease::from_str(&as_string)
    }
}

impl TryFrom<&Version> for NightlyPrerelease {
    type Error = anyhow::Error;

    #[context("Failed to parse nightly version: `{value}`.")]
    fn try_from(value: &Version) -> std::result::Result<Self, Self::Error> {
        Self::try_from(&value.pre)
    }
}

impl NightlyPrerelease {
    /// Get the date of the nightly build.
    pub fn date(&self) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd(self.year as i32, self.month as u32, self.day as u32)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nightly {
    full_version: Version,
    prerelease:   NightlyPrerelease,
}

impl Nightly {
    pub fn new(full_version: Version) -> anyhow::Result<Self> {
        let prerelease = NightlyPrerelease::try_from(&full_version)?;
        Ok(Self { full_version, prerelease })
    }

    pub fn next(&self) -> Self {
        let prerelease = self.prerelease.next();
        let full_version =
            Version { pre: prerelease.try_into().unwrap(), ..self.full_version.clone() };
        Self { full_version, prerelease }
    }

    /// Get as [`Version`].
    pub fn version(&self) -> &Version {
        &self.full_version
    }

    /// Get the parser prerelease piece of the version.
    pub fn prerelease(&self) -> &NightlyPrerelease {
        &self.prerelease
    }
}

impl TryFrom<&Version> for Nightly {
    type Error = anyhow::Error;

    #[context("Failed to parse nightly version: `{value}`.")]
    fn try_from(value: &Version) -> std::result::Result<Self, Self::Error> {
        let prerelease = NightlyPrerelease::try_from(value)?;
        Ok(Self { full_version: value.clone(), prerelease })
    }
}

impl From<Nightly> for Version {
    fn from(value: Nightly) -> Self {
        value.full_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn parsing_nightly_prerelease() -> Result {
        let nightly = Version::from_str("2020.1.1-nightly.2020.12.31")?;
        assert_eq!(NightlyPrerelease::try_from(&nightly)?, NightlyPrerelease {
            year:  2020,
            month: 12,
            day:   31,
            index: None,
        });
        let nightly_with_index = Version::from_str("2020.1.1-nightly.2020.12.31.1")?;
        assert_eq!(NightlyPrerelease::try_from(&nightly_with_index)?, NightlyPrerelease {
            year:  2020,
            month: 12,
            day:   31,
            index: Some(1),
        });

        let nightly_with_too_many_identifiers =
            Version::from_str("2020.1.1-nightly.2020.12.31.1.1")?;
        assert!(NightlyPrerelease::try_from(&nightly_with_too_many_identifiers).is_err());

        let nightly_with_missing_year = Version::from_str("2020.1.1-nightly.12.31")?;
        assert!(NightlyPrerelease::try_from(&nightly_with_missing_year).is_err());

        let non_nightly = Version::from_str("2020.1.1")?;
        assert!(NightlyPrerelease::try_from(&non_nightly).is_err());

        let rc = Version::from_str("2020.1.1-rc.1")?;
        assert!(NightlyPrerelease::try_from(&rc).is_err());
        Ok(())
    }
}
