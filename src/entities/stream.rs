//! Owned live ChemApp stream.

use crate::calculator::Calculator;
use crate::error::ChemAppError;
use crate::snapshot::StreamSnapshot;

/// Tracks whether this Rust handle still owns native cleanup. Keeping this
/// small state separate makes explicit removal and best-effort `Drop`
/// idempotent without inventing shared native ownership.
#[derive(Debug)]
struct StreamLease {
    state: StreamLeaseState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StreamLeaseState {
    Active,
    Removing,
    Released,
}

impl StreamLease {
    fn active() -> Self {
        Self {
            state: StreamLeaseState::Active,
        }
    }

    /// Consume Drop's cleanup responsibility before an observable native call.
    fn begin_explicit_removal(&mut self) -> bool {
        if self.state == StreamLeaseState::Active {
            self.state = StreamLeaseState::Removing;
            true
        } else {
            false
        }
    }

    fn complete_explicit_removal(&mut self) {
        debug_assert_eq!(self.state, StreamLeaseState::Removing);
        self.state = StreamLeaseState::Released;
    }

    /// Returns whether Drop owns exactly one best-effort cleanup call.
    fn begin_drop_cleanup(&mut self) -> bool {
        if self.state == StreamLeaseState::Active {
            self.state = StreamLeaseState::Released;
            true
        } else {
            false
        }
    }
}

/// The sole high-level owner of one name-addressed native ChemApp stream.
///
/// The manual defines streams and their removal by `IDENTS`, but does not
/// specify duplicate `TQSTTP` creation behavior. `Calculator` therefore
/// leases each name to at most one live `Stream`; direct `Engine` calls remain
/// an intentional low-level escape hatch outside this ownership guarantee.
pub struct Stream<'a> {
    pub(crate) calculator: &'a Calculator,
    name: String,
    temp: f64,
    pres: f64,
    lease: StreamLease,
}

impl<'a> Stream<'a> {
    pub fn new(
        calculator: &'a Calculator,
        name: &str,
        temp: f64,
        pres: f64,
    ) -> Result<Self, ChemAppError> {
        calculator.claim_stream_name(name)?;
        if let Err(error) = calculator.engine.tqsttp(name, (temp, pres)) {
            calculator.release_stream_name(name);
            return Err(error);
        }
        Ok(Self {
            calculator,
            name: name.to_owned(),
            temp,
            pres,
            lease: StreamLease::active(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn temperature(&self) -> f64 {
        self.temp
    }
    pub fn pressure(&self) -> f64 {
        self.pres
    }

    pub fn snapshot(&self) -> Result<StreamSnapshot, ChemAppError> {
        StreamSnapshot::new(self)
    }

    pub fn table_string(&self) -> Result<String, ChemAppError> {
        crate::table::live_stream_table(self)
    }

    /// Removes this stream and consumes its unique high-level ownership.
    ///
    /// Cleanup responsibility is consumed before TQSTRM. Consequently a
    /// native error remains observable and Drop never makes a hidden retry.
    /// On failure the name remains reserved because native stream state is
    /// unknown; callers must not create a second high-level owner for it.
    pub fn remove(mut self) -> Result<(), ChemAppError> {
        debug_assert!(self.lease.begin_explicit_removal());
        match self.calculator.engine.tqstrm(&self.name) {
            Ok(()) => {
                self.lease.complete_explicit_removal();
                self.calculator.release_stream_name(&self.name);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn add_with_indices(
        &self,
        indexp: usize,
        indexc: usize,
        val: f64,
    ) -> Result<(), ChemAppError> {
        self.calculator
            .engine
            .tqstca(&self.name, indexp, indexc, val)
    }

    pub fn add_with_names(
        &self,
        phase: &str,
        constituent: &str,
        val: f64,
    ) -> Result<(), ChemAppError> {
        let indexp = self.calculator.engine.tqinp(phase)?;
        let indexc = self.calculator.engine.tqinpc(indexp, constituent)?;
        self.add_with_indices(indexp, indexc, val)
    }

    fn property(&self, option: &str) -> Result<f64, ChemAppError> {
        self.calculator.engine.tqstxp(&self.name, option)
    }

    pub fn cp(&self) -> Result<f64, ChemAppError> {
        self.property("CP")
    }
    pub fn h(&self) -> Result<f64, ChemAppError> {
        self.property("H")
    }
    pub fn s(&self) -> Result<f64, ChemAppError> {
        self.property("S")
    }
    pub fn g(&self) -> Result<f64, ChemAppError> {
        self.property("G")
    }
    pub fn v(&self) -> Result<f64, ChemAppError> {
        self.property("V")
    }
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        if self.lease.begin_drop_cleanup() {
            // Drop cannot report native cleanup failures. Releasing the Rust
            // lease is still required because the value is gone; callers that
            // need to observe cleanup errors should use consuming `remove`.
            let _ = self.calculator.engine.tqstrm(&self.name);
            self.calculator.release_stream_name(&self.name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_removal_disables_drop_cleanup_idempotently() {
        let mut lease = StreamLease::active();
        assert!(lease.begin_explicit_removal());
        assert!(!lease.begin_drop_cleanup());
        lease.complete_explicit_removal();
        assert!(!lease.begin_drop_cleanup());
    }

    #[test]
    fn failed_explicit_removal_never_rearms_drop_cleanup() {
        let mut lease = StreamLease::active();
        assert!(lease.begin_explicit_removal());
        // Model a native error by deliberately not completing removal.
        assert_eq!(lease.state, StreamLeaseState::Removing);
        assert!(!lease.begin_drop_cleanup());
    }

    #[test]
    fn ordinary_drop_gets_exactly_one_cleanup_attempt() {
        let mut lease = StreamLease::active();
        assert!(lease.begin_drop_cleanup());
        assert!(!lease.begin_drop_cleanup());
    }
}
